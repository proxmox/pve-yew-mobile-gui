use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_human_byte::HumanByte;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::widget::menu::{Menu, MenuItem, SplitButton};
use pwt::widget::{
    Button, Column, ConfirmDialog, Fa, List, ListTile, MiniScroll, MiniScrollMode, Row,
};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::utils::lookup_task_description;
use proxmox_yew_comp::{
    http_get, http_post, percent_encoding::percent_encode_component, ConsoleType, XTermJs,
};

use pve_api_types::{IsRunning, LxcStatus};

use crate::widgets::{icon_list_tile, list_tile_usage, standard_list_tile, TasksListButton};

use super::LxcResourcesPanel;

#[derive(Clone, PartialEq, Properties)]
pub struct LxcDashboardPanel {
    vmid: u32,
    node: AttrValue,
}

impl LxcDashboardPanel {
    pub fn new(node: impl Into<AttrValue>, vmid: u32) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

pub struct PveLxcDashboardPanel {
    data: Option<Result<LxcStatus, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    cmd_guard: Option<AsyncAbortGuard>,
    running_upid: Option<String>,
    confirm_lxc_command: Option<(String, String)>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ConfirmableCommands {
    Reboot,
    Stop,
    Shutdown,
}

pub enum Msg {
    Load,
    LoadResult(Result<LxcStatus, Error>),
    CommandResult(Result<String, Error>),
    LxcCommand(String),
    Confirm(ConfirmableCommands),
    CloseDialog,
}

fn get_status_url(node: &str, vmid: u32, cmd: &str) -> String {
    format!(
        "/nodes/{}/lxc/{}/status/{cmd}",
        percent_encode_component(node),
        vmid
    )
}

fn large_fa_icon(name: &str, running: bool) -> Fa {
    Fa::new(name)
        .fixed_width()
        .class("pwt-font-size-title-large")
        .class(running.then(|| "pwt-color-primary"))
}

fn format_guest_task_confirmation(
    command: ConfirmableCommands,
    vmid: u32,
    guest_name: Option<&str>,
) -> String {
    let task_type = match command {
        ConfirmableCommands::Reboot => "vzreboot",
        ConfirmableCommands::Shutdown => "vzshutdown",
        ConfirmableCommands::Stop => "vzstop",
    };
    let lxc_name_or_id = match guest_name {
        Some(name) => name.to_string(),
        None => vmid.to_string(),
    };
    lookup_task_description(task_type, Some(&lxc_name_or_id)).unwrap()
}

impl PveLxcDashboardPanel {
    fn lxc_command(&mut self, ctx: &Context<Self>, cmd: &str) {
        let props = ctx.props();
        let url = get_status_url(&props.node, props.vmid, cmd);
        let link = ctx.link().clone();
        self.cmd_guard = Some(AsyncAbortGuard::spawn(async move {
            let result = http_post(&url, None).await;
            link.send_message(Msg::CommandResult(result));
        }));
    }

    fn confirmed_lxc_command(&mut self, ctx: &Context<Self>, command: ConfirmableCommands) {
        let props = ctx.props();
        let guest_name = match &self.data {
            Some(Ok(data)) => data.name.as_deref(),
            _ => None,
        };
        let confirm_msg = format_guest_task_confirmation(command, props.vmid, guest_name);

        let command_str = match command {
            ConfirmableCommands::Reboot => "reboot",
            ConfirmableCommands::Shutdown => "shutdown",
            ConfirmableCommands::Stop => "stop",
        };

        self.confirm_lxc_command = Some((command_str.into(), confirm_msg));
    }

    fn view_status(&self, ctx: &Context<Self>, data: &LxcStatus) -> Html {
        let props = ctx.props();

        let ct_icon = large_fa_icon("cube", data.status == IsRunning::Running);

        let mut tiles: Vec<ListTile> = Vec::new();

        tiles.push(standard_list_tile(
            format!("{} {}", data.vmid, data.name.as_deref().unwrap_or("")),
            &props.node,
            Some(ct_icon.clone().into()),
            Some(data.status.to_string().into()),
        ));

        if let Some(Ok(data)) = &self.data {
            if data.status == IsRunning::Running {
                if let (Some(cpu), Some(maxcpu)) = (data.cpu, data.cpus) {
                    let cpu_percentage = if maxcpu == 0.0 {
                        0.0
                    } else {
                        (cpu as f32) / (maxcpu as f32)
                    };

                    tiles.push(
                        icon_list_tile(Fa::new("cpu"), "CPU", None::<&str>, None).with_child(
                            list_tile_usage(
                                format!("{:.2}", cpu),
                                maxcpu.to_string(),
                                cpu_percentage,
                            ),
                        ),
                    );
                }

                if let (Some(mem), Some(maxmem)) = (data.mem, data.maxmem) {
                    let mem_percentage = if maxmem <= 0 {
                        0.0
                    } else {
                        (mem as f32) / (maxmem as f32)
                    };
                    tiles.push(
                        icon_list_tile(Fa::new("memory"), "Memory", None::<&str>, None).with_child(
                            list_tile_usage(
                                HumanByte::new_binary(mem as f64).to_string(),
                                HumanByte::new_binary(maxmem as f64).to_string(),
                                mem_percentage,
                            ),
                        ),
                    );
                }
            }
        }

        let status = List::new(tiles.len() as u64, move |pos| {
            tiles[pos as usize].clone().into()
        })
        .grid_template_columns("auto 1fr auto");

        crate::widgets::standard_card(tr!("Status"), None::<&str>)
            .with_child(status)
            .into()
    }

    fn view_actions(&self, ctx: &Context<Self>, data: &LxcStatus) -> Html {
        let props = ctx.props();
        let vmid = props.vmid;
        let node_name = props.node.clone();

        let running = data.status == IsRunning::Running;

        let menu = Menu::new()
            .with_item(
                MenuItem::new(tr!("Reboot")).disabled(!running).on_select(
                    ctx.link()
                        .callback(|_| Msg::Confirm(ConfirmableCommands::Reboot)),
                ),
            )
            .with_item(
                MenuItem::new(tr!("Stop")).disabled(!running).on_select(
                    ctx.link()
                        .callback(|_| Msg::Confirm(ConfirmableCommands::Stop)),
                ),
            );

        let shutdown = SplitButton::new(tr!("Shutdown"))
            .disabled(!running)
            .menu(menu)
            .on_activate(
                ctx.link()
                    .callback(|_| Msg::Confirm(ConfirmableCommands::Shutdown)),
            );

        let row = Row::new()
            .padding_y(1)
            .gap(2)
            .class(pwt::css::JustifyContent::SpaceBetween)
            .with_child(
                Button::new(tr!("Start"))
                    .disabled(running)
                    .on_activate(ctx.link().callback(|_| Msg::LxcCommand("start".into()))),
            )
            .with_child(shutdown)
            .with_child(
                Button::new(tr!("Console"))
                    .icon_class("fa fa-terminal")
                    .on_activate(move |_| {
                        XTermJs::open_xterm_js_viewer(
                            ConsoleType::LXC(vmid.into()),
                            &node_name,
                            true,
                        );
                    }),
            );

        MiniScroll::new(row)
            .scroll_mode(MiniScrollMode::Native)
            .class(pwt::css::Flex::None)
            .into()
    }

    fn task_button(&self, ctx: &Context<Self>) -> Html {
        TasksListButton::new()
            .running_upid(self.running_upid.clone())
            .on_show_task_list({
                let navigator = ctx.link().navigator().clone().unwrap();
                let props = ctx.props();
                let node = props.node.clone();
                let vmid = props.vmid;
                move |_| {
                    navigator.push(&crate::Route::LxcTasks {
                        vmid,
                        nodename: node.to_string(),
                    });
                }
            })
            .into()
    }
}

impl Component for PveLxcDashboardPanel {
    type Message = Msg;
    type Properties = LxcDashboardPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            reload_timeout: None,
            load_guard: None,
            cmd_guard: None,
            running_upid: None,
            confirm_lxc_command: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Load => {
                let link = ctx.link().clone();
                let url = get_status_url(&props.node, props.vmid, "current");
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.data = Some(result.map_err(|err| err.to_string()));
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
            Msg::CommandResult(result) => match result {
                Ok(upid) => {
                    self.running_upid = Some(upid);
                }
                Err(err) => {
                    self.running_upid = None;
                    crate::show_failed_command_error(ctx.link(), err);
                }
            },
            Msg::Confirm(command) => self.confirmed_lxc_command(ctx, command),
            Msg::LxcCommand(command) => self.lxc_command(ctx, &command),
            Msg::CloseDialog => self.confirm_lxc_command = None,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        crate::widgets::render_loaded_data(&self.data, |data| {
            let confirm_dialog = self
                .confirm_lxc_command
                .as_ref()
                .map(|(command, confirm_msg)| {
                    ConfirmDialog::default()
                        .confirm_message(confirm_msg)
                        .on_close(ctx.link().callback(|_| Msg::CloseDialog))
                        .on_confirm({
                            let command = command.clone();
                            ctx.link()
                                .callback(move |_| Msg::LxcCommand(command.clone()))
                        })
                });

            Column::new()
                .class(pwt::css::FlexFit)
                .padding(2)
                .gap(2)
                .with_child(self.view_status(ctx, data))
                .with_child(self.view_actions(ctx, data))
                .with_child(self.task_button(ctx))
                .with_child(LxcResourcesPanel::new(props.node.clone(), props.vmid))
                .with_optional_child(confirm_dialog)
                .into()
        })
    }
}

impl Into<VNode> for LxcDashboardPanel {
    fn into(self) -> VNode {
        let comp = VComp::new::<PveLxcDashboardPanel>(Rc::new(self), None);
        VNode::from(comp)
    }
}

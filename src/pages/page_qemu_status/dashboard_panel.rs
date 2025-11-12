use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_human_byte::HumanByte;

use serde_json::{json, Value};
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::widget::menu::{Menu, MenuItem, SplitButton};
use pwt::widget::{
    Button, Column, ConfirmDialog, Fa, List, ListTile, MiniScroll, MiniScrollMode, Row,
};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::configuration::pve::QemuHardwarePanel;
use proxmox_yew_comp::layout::card::standard_card;
use proxmox_yew_comp::layout::list_tile::{icon_list_tile, list_tile_usage, standard_list_tile};
use proxmox_yew_comp::layout::render_loaded_data;
use proxmox_yew_comp::utils::lookup_task_description;
use proxmox_yew_comp::{
    http_get, http_post, percent_encoding::percent_encode_component, ConsoleType, XTermJs,
};

use pve_api_types::{IsRunning, QemuStatus};

use crate::widgets::TasksListButton;

#[derive(Clone, PartialEq, Properties)]
pub struct QemuDashboardPanel {
    vmid: u32,
    node: AttrValue,
}

impl QemuDashboardPanel {
    pub fn new(node: impl Into<AttrValue>, vmid: u32) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

pub struct PveQemuDashboardPanel {
    data: Option<Result<QemuStatus, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    cmd_guard: Option<AsyncAbortGuard>,
    running_upid: Option<String>,
    confirm_vm_command: Option<(String, String, Option<Value>)>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ConfirmableCommands {
    Pause,
    Reboot,
    Reset,
    Suspend,
    Stop,
    Shutdown,
}

pub enum Msg {
    Load,
    LoadResult(Result<QemuStatus, Error>),
    CommandResult(Result<String, Error>),
    StartCommand(String),
    Confirm(ConfirmableCommands),
    CloseDialog,
    VmCommand((String, Option<Value>)),
}

fn get_status_url(node: &str, vmid: u32, cmd: &str) -> String {
    format!(
        "/nodes/{}/qemu/{}/status/{cmd}",
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
        ConfirmableCommands::Pause => "qmpause",
        ConfirmableCommands::Suspend => "qmsuspend",
        ConfirmableCommands::Reset => "qmreset",
        ConfirmableCommands::Reboot => "qmreboot",
        ConfirmableCommands::Shutdown => "qmshutdown",
        ConfirmableCommands::Stop => "qmstop",
    };
    let vm_name_or_id = match guest_name {
        Some(name) => name.to_string(),
        None => vmid.to_string(),
    };
    lookup_task_description(task_type, Some(&vm_name_or_id)).unwrap()
}

impl PveQemuDashboardPanel {
    fn vm_command(&mut self, ctx: &Context<Self>, cmd: &str, param: Option<Value>) {
        let props = ctx.props();
        let url = get_status_url(&props.node, props.vmid, cmd);
        let link = ctx.link().clone();
        self.cmd_guard = Some(AsyncAbortGuard::spawn(async move {
            let result = http_post(&url, param.clone()).await;
            link.send_message(Msg::CommandResult(result));
        }));
    }

    fn confirmed_vm_command(&mut self, ctx: &Context<Self>, command: ConfirmableCommands) {
        let props = ctx.props();
        let guest_name = match &self.data {
            Some(Ok(data)) => data.name.as_deref(),
            _ => None,
        };
        let confirm_msg = format_guest_task_confirmation(command, props.vmid, guest_name);

        let (command_str, param) = match command {
            ConfirmableCommands::Pause => ("suspend", None),
            ConfirmableCommands::Suspend => ("suspend", Some(json!({ "todisk": true }))),
            ConfirmableCommands::Reset => ("reset", None),
            ConfirmableCommands::Reboot => ("reboot", None),
            ConfirmableCommands::Shutdown => ("shutdown", None),
            ConfirmableCommands::Stop => ("stop", None),
        };

        self.confirm_vm_command = Some((command_str.into(), confirm_msg, param));
    }

    fn view_status(&self, ctx: &Context<Self>, data: &QemuStatus) -> Html {
        let props = ctx.props();

        let vm_icon: Html = large_fa_icon("desktop", data.status == IsRunning::Running).into();

        let mut tiles: Vec<ListTile> = Vec::new();

        tiles.push(standard_list_tile(
            format!("{} {}", data.vmid, data.name.as_deref().unwrap_or("")),
            props.node.clone(),
            vm_icon,
            data.qmpstatus.clone().unwrap_or(data.status.to_string()),
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
                        icon_list_tile(Fa::new("cpu"), tr!("CPU"), (), ()).with_child(
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
                        icon_list_tile(Fa::new("memory"), tr!("Memory"), (), ()).with_child(
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

        let status = List::from_tiles(tiles).grid_template_columns("auto 1fr auto");

        standard_card(tr!("Status"), (), ())
            .with_child(status)
            .into()
    }

    fn view_actions(&self, ctx: &Context<Self>, data: &QemuStatus) -> Html {
        let props = ctx.props();
        let vmid = props.vmid;
        let node_name = props.node.clone();

        let qmpstatus = data.qmpstatus.as_deref().unwrap_or("");
        let running = data.status == IsRunning::Running;

        let menu = Menu::new()
            .with_item(
                MenuItem::new(tr!("Reboot")).disabled(!running).on_select(
                    ctx.link()
                        .callback(|_| Msg::Confirm(ConfirmableCommands::Reboot)),
                ),
            )
            .with_item(
                MenuItem::new(tr!("Pause")).disabled(!running).on_select(
                    ctx.link()
                        .callback(|_| Msg::Confirm(ConfirmableCommands::Pause)),
                ),
            )
            .with_item(
                MenuItem::new(tr!("Hibernate"))
                    .disabled(!running)
                    .on_select(
                        ctx.link()
                            .callback(|_| Msg::Confirm(ConfirmableCommands::Suspend)),
                    ),
            )
            .with_item(
                MenuItem::new(tr!("Stop")).disabled(!running).on_select(
                    ctx.link()
                        .callback(|_| Msg::Confirm(ConfirmableCommands::Stop)),
                ),
            )
            .with_item(
                MenuItem::new(tr!("Reset")).disabled(!running).on_select(
                    ctx.link()
                        .callback(|_| Msg::Confirm(ConfirmableCommands::Reset)),
                ),
            );

        let shutdown = SplitButton::new(tr!("Shutdown"))
            .disabled(!running)
            .menu(menu)
            .on_activate(
                ctx.link()
                    .callback(|_| Msg::Confirm(ConfirmableCommands::Shutdown)),
            );

        let resume = ["prelaunch", "paused", "suspended"].contains(&qmpstatus);

        let row = Row::new()
            .padding_y(1)
            .gap(2)
            .class(pwt::css::JustifyContent::SpaceBetween)
            .with_child(if resume {
                Button::new(tr!("Resume")).on_activate(
                    ctx.link()
                        .callback(|_| Msg::VmCommand(("resume".into(), None))),
                )
            } else {
                Button::new(tr!("Start")).disabled(running).on_activate(
                    ctx.link()
                        .callback(|_| Msg::VmCommand(("start".into(), None))),
                )
            })
            .with_child(shutdown)
            .with_child(
                Button::new(tr!("Console"))
                    .icon_class("fa fa-terminal")
                    .on_activate(move |_| {
                        XTermJs::open_xterm_js_viewer(
                            ConsoleType::KVM(vmid.into()),
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
                    navigator.push(&crate::Route::QemuTasks {
                        vmid,
                        nodename: node.to_string(),
                    });
                }
            })
            .into()
    }
}

impl Component for PveQemuDashboardPanel {
    type Message = Msg;
    type Properties = QemuDashboardPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            reload_timeout: None,
            load_guard: None,
            cmd_guard: None,
            running_upid: None,
            confirm_vm_command: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::CloseDialog => {
                self.confirm_vm_command = None;
            }
            Msg::Load => {
                let link = ctx.link().clone();
                let url = get_status_url(&props.node, props.vmid, "current");
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result: Result<Value, _> = http_get(&url, None).await;
                    let result = match result {
                        Ok(mut data) => {
                            // hack: The PVE api sometimes return Null for diskread/diskwrite
                            // so we simply remove Null values...
                            if let Value::Object(map) = &mut data {
                                map.retain(|_k, v| v != &Value::Null);
                            }
                            let data = serde_json::from_value::<QemuStatus>(data)
                                .map_err(|err| err.into());
                            data
                        }
                        Err(err) => Err(err),
                    };
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
            Msg::VmCommand((ref command_str, ref param)) => {
                self.vm_command(ctx, command_str, param.clone())
            }
            Msg::StartCommand(upid) => self.running_upid = Some(upid),
            Msg::CommandResult(result) => match result {
                Ok(upid) => {
                    self.running_upid = Some(upid);
                }
                Err(err) => {
                    self.running_upid = None;
                    crate::show_failed_command_error(ctx.link(), err);
                }
            },
            Msg::Confirm(command) => self.confirmed_vm_command(ctx, command),
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        render_loaded_data(&self.data, |data| {
            let confirm_dialog =
                self.confirm_vm_command
                    .as_ref()
                    .map(|(command, confirm_msg, param)| {
                        ConfirmDialog::default()
                            .confirm_message(confirm_msg)
                            .on_close(ctx.link().callback(|_| Msg::CloseDialog))
                            .on_confirm({
                                let command = command.clone();
                                let param = param.clone();
                                ctx.link().callback(move |_| {
                                    Msg::VmCommand((command.clone(), param.clone()))
                                })
                            })
                    });

            Column::new()
                .class(pwt::css::FlexFit)
                .padding(2)
                .gap(2)
                .with_child(self.view_status(ctx, data))
                .with_child(self.view_actions(ctx, data))
                .with_child(self.task_button(ctx))
                .with_child(
                    QemuHardwarePanel::new(props.node.clone(), props.vmid)
                        .mobile(true)
                        //.readonly(true)
                        .on_start_command(ctx.link().callback(Msg::StartCommand)),
                )
                .with_optional_child(confirm_dialog)
                .into()
        })
    }
}

impl Into<VNode> for QemuDashboardPanel {
    fn into(self) -> VNode {
        let comp = VComp::new::<PveQemuDashboardPanel>(Rc::new(self), None);
        VNode::from(comp)
    }
}

use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_human_byte::HumanByte;
use serde::{Deserialize, Serialize};

use pwt::props::StorageLocation;
use pwt::state::PersistentState;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::widget::menu::{Menu, MenuItem, SplitButton};
use pwt::widget::{
    Button, Card, Column, Container, Fa, List, ListTile, MiniScroll, MiniScrollMode, Row, TabBar,
    TabBarItem,
};
use pwt::AsyncPool;

use proxmox_yew_comp::{http_get, http_post, percent_encoding::percent_encode_component};

use pve_api_types::{IsRunning, QemuStatus};

use crate::widgets::{
    icon_list_tile, list_tile_usage, standard_list_tile, TopNavBar, VmConfigPanel, VmHardwarePanel,
};

#[derive(Clone, PartialEq, Properties)]
pub struct PageVmStatus {
    vmid: u32,
    node: AttrValue,
}

impl PageVmStatus {
    pub fn new(node: impl Into<AttrValue>, vmid: u32) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum ViewState {
    #[default]
    Dashboard,
    Options,
    Backup,
}
pub struct PvePageVmStatus {
    view_state: PersistentState<ViewState>,
    data: Result<QemuStatus, String>,
    reload_timeout: Option<Timeout>,
    async_pool: AsyncPool,
}

pub enum Msg {
    Load,
    LoadResult(Result<QemuStatus, Error>),
    CommandResult(Result<String, Error>),
    Start,
    Stop,
    Shutdown,
    SetViewState(ViewState),
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

impl PvePageVmStatus {
    fn vm_command(&self, ctx: &Context<Self>, cmd: &str) {
        let props = ctx.props();
        let url = get_status_url(&props.node, props.vmid, cmd);
        let link = ctx.link().clone();
        self.async_pool.spawn(async move {
            let result = http_post(&url, None).await;
            link.send_message(Msg::CommandResult(result));
        });
    }

    fn view_status(&self, ctx: &Context<Self>, data: &QemuStatus) -> Html {
        let props = ctx.props();

        let vm_icon = large_fa_icon("desktop", data.status == IsRunning::Running);

        let mut tiles: Vec<ListTile> = Vec::new();

        tiles.push(standard_list_tile(
            format!("{} {}", data.vmid, data.name.as_deref().unwrap_or("")),
            &props.node,
            Some(vm_icon.clone().into()),
            Some(data.status.to_string().into()),
        ));

        if let Ok(data) = &self.data {
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

    fn view_actions(&self, ctx: &Context<Self>, data: &QemuStatus) -> Html {
        let running = data.status == IsRunning::Running;

        let menu = Menu::new().with_item(
            MenuItem::new("Stop")
                .disabled(!running)
                .on_select(ctx.link().callback(|_| Msg::Stop)),
        );

        let shutdown = SplitButton::new("Shutdown")
            .disabled(!running)
            .menu(menu)
            .on_activate(ctx.link().callback(|_| Msg::Shutdown));

        let row = Row::new()
            .padding_y(1)
            .gap(2)
            .class(pwt::css::JustifyContent::SpaceBetween)
            .with_child(
                Button::new("Start")
                    .disabled(running)
                    .on_activate(ctx.link().callback(|_| Msg::Start)),
            )
            .with_child(shutdown)
            .with_child(Button::new("Console").disabled(!running));

        MiniScroll::new(row)
            .scroll_mode(MiniScrollMode::Native)
            .class(pwt::css::Flex::None)
            .into()
    }

    fn task_button(&self, ctx: &Context<Self>) -> Html {
        Card::new()
            .padding(2)
            .class("pwt-d-flex")
            .class("pwt-interactive")
            .class(pwt::css::JustifyContent::Center)
            .with_child("Task List")
            .onclick({
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

    fn view_dashboard(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        match &self.data {
            Ok(data) => Column::new()
                .class(pwt::css::FlexFit)
                .padding(2)
                .gap(2)
                .with_child(self.view_status(ctx, data))
                .with_child(self.view_actions(ctx, data))
                .with_child(self.task_button(ctx))
                .with_child(VmHardwarePanel::new(props.node.clone(), props.vmid))
                .into(),
            Err(err) => pwt::widget::error_message(err).into(),
        }
    }

    fn view_backup(&self, _ctx: &Context<Self>) -> Html {
        Container::new().with_child("Backup").into()
    }
}

impl Component for PvePageVmStatus {
    type Message = Msg;
    type Properties = PageVmStatus;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        ctx.link().send_message(Msg::Load);

        let view_state = PersistentState::new(StorageLocation::session(format!(
            "vm-{}-status-tab-bar-state",
            props.vmid
        )));

        Self {
            data: Err(format!("no data loaded")),
            reload_timeout: None,
            async_pool: AsyncPool::new(),
            view_state,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Load => {
                let link = ctx.link().clone();
                let url = get_status_url(&props.node, props.vmid, "current");
                self.async_pool.spawn(async move {
                    let result = http_get(&url, None).await;
                    link.send_message(Msg::LoadResult(result));
                });
            }
            Msg::LoadResult(result) => {
                self.data = result.map_err(|err| err.to_string());
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
            Msg::CommandResult(result) => {
                log::info!("Result {:?}", result);
            }
            Msg::Start => self.vm_command(ctx, "start"),
            Msg::Stop => self.vm_command(ctx, "stop"),
            Msg::Shutdown => self.vm_command(ctx, "shutdown"),
            Msg::SetViewState(view_state) => {
                self.view_state.update(view_state);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (active_tab, content) = match *self.view_state {
            ViewState::Dashboard => ("dashboard", self.view_dashboard(ctx)),
            ViewState::Backup => ("backup", self.view_backup(ctx)),
            ViewState::Options => (
                "options",
                VmConfigPanel::new(props.node.clone(), props.vmid).into(),
            ),
        };

        let tab_bar = TabBar::new()
            .class(pwt::css::JustifyContent::Center)
            .active(active_tab)
            .with_item(
                TabBarItem::new()
                    .label("Dashboard")
                    .key("dashboard")
                    .on_activate(
                        ctx.link()
                            .callback(|_| Msg::SetViewState(ViewState::Dashboard)),
                    ),
            )
            .with_item(
                TabBarItem::new()
                    .label("Options")
                    .key("options")
                    .on_activate(
                        ctx.link()
                            .callback(|_| Msg::SetViewState(ViewState::Options)),
                    ),
            )
            .with_item(
                TabBarItem::new().label("Backup").key("backup").on_activate(
                    ctx.link()
                        .callback(|_| Msg::SetViewState(ViewState::Backup)),
                ),
            );

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("VM {}", props.vmid))
                    .back(true),
            )
            .with_child(tab_bar)
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageVmStatus {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageVmStatus>(Rc::new(self), None);
        VNode::from(comp)
    }
}

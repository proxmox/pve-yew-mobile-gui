use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_human_byte::HumanByte;

use proxmox_yew_comp::{ConfirmButton, ConsoleType};
use serde_json::json;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::touch::{SnackBar, SnackBarContextExt};
use pwt::widget::{Button, Column, Fa, List, ListTile, MiniScroll, MiniScrollMode, Progress, Row};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, http_post, percent_encoding::percent_encode_component, XTermJs};

use pve_api_types::NodeStatus;

use crate::widgets::{icon_list_tile, list_tile_usage, TasksListButton};

//use super::NodeResourcesPanel;

#[derive(Clone, PartialEq, Properties)]
pub struct NodeDashboardPanel {
    node: AttrValue,
}

impl NodeDashboardPanel {
    pub fn new(node: impl Into<AttrValue>) -> Self {
        Self { node: node.into() }
    }
}

pub struct PveNodeDashboardPanel {
    data: Option<Result<NodeStatus, String>>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    cmd_guard: Option<AsyncAbortGuard>,
}

pub enum Msg {
    Load,
    LoadResult(Result<NodeStatus, Error>),
    CommandResult(Result<(), Error>),
    Reboot,
    Shutdown,
}

fn get_status_url(node: &str) -> String {
    format!("/nodes/{}/status", percent_encode_component(node),)
}

impl PveNodeDashboardPanel {
    fn node_command(&mut self, ctx: &Context<Self>, cmd: &str) {
        let props = ctx.props();
        let url = get_status_url(&props.node);
        let link = ctx.link().clone();
        let param = json!({ "command": cmd});
        self.cmd_guard = Some(AsyncAbortGuard::spawn(async move {
            let result = http_post(&url, Some(param.clone())).await;
            link.send_message(Msg::CommandResult(result));
        }));
    }

    fn view_status(&self, _ctx: &Context<Self>, data: &NodeStatus) -> Html {
        let mut tiles: Vec<ListTile> = Vec::new();

        tiles.push(
            icon_list_tile(
                Fa::new("cpu"),
                format!(
                    "CPU ({} Cores, {} Sockets)",
                    data.cpuinfo.cores, data.cpuinfo.sockets
                ),
                data.cpuinfo.model.clone(),
                None,
            )
            .with_child(list_tile_usage(
                format!("{:.2}", data.cpu * 100.0) + "%",
                format!("% of {} threads", data.cpuinfo.cpus), // fixme
                data.cpu as f32,
            )),
        );

        let mem_percentage = if data.memory.total <= 0 {
            0.0
        } else {
            (data.memory.used as f32) / (data.memory.total as f32)
        };
        tiles.push(
            icon_list_tile(Fa::new("memory"), tr!("Memory"), None::<&str>, None).with_child(
                list_tile_usage(
                    HumanByte::new_binary(data.memory.used as f64).to_string(),
                    HumanByte::new_binary(data.memory.total as f64).to_string(),
                    mem_percentage,
                ),
            ),
        );
        let rootfs_percentage = if data.rootfs.total <= 0 {
            0.0
        } else {
            (data.rootfs.used as f32) / (data.rootfs.total as f32)
        };
        tiles.push(
            icon_list_tile(Fa::new("hdd-o"), tr!("Root Filesystem"), None::<&str>, None)
                .with_child(list_tile_usage(
                    HumanByte::new_binary(data.rootfs.used as f64).to_string(),
                    HumanByte::new_binary(data.rootfs.total as f64).to_string(),
                    rootfs_percentage,
                )),
        );

        let status = List::new(tiles.len() as u64, move |pos| {
            tiles[pos as usize].clone().into()
        })
        .grid_template_columns("auto 1fr auto");

        crate::widgets::standard_card(tr!("Summary"), data.pveversion.clone())
            .with_child(status)
            .into()
    }

    fn view_actions(&self, ctx: &Context<Self>, _data: &NodeStatus) -> Html {
        let props = ctx.props();
        let node_name = props.node.clone();

        let row = Row::new()
            .padding_y(1)
            .gap(2)
            .class(pwt::css::JustifyContent::SpaceBetween)
            .with_child(
                ConfirmButton::new(tr!("Reboot"))
                    .confirm_message(tr!("Reboot node '{0}'?", props.node))
                    .on_activate(ctx.link().callback(|_| Msg::Reboot)),
            )
            .with_child(
                ConfirmButton::new(tr!("Shutdown"))
                    .confirm_message(tr!("Shutdown node '{0}'?", props.node))
                    .on_activate(ctx.link().callback(|_| Msg::Shutdown)),
            )
            .with_child(
                Button::new("Console")
                    .icon_class("fa fa-terminal")
                    .on_activate(move |_| {
                        XTermJs::open_xterm_js_viewer(ConsoleType::LoginShell, &node_name, true);
                    }),
            );

        MiniScroll::new(row)
            .scroll_mode(MiniScrollMode::Native)
            .class(pwt::css::Flex::None)
            .into()
    }

    fn task_button(&self, ctx: &Context<Self>) -> Html {
        TasksListButton::new()
            .on_show_task_list({
                let navigator = ctx.link().navigator().clone().unwrap();
                let props = ctx.props();
                let node = props.node.clone();
                move |_| {
                    navigator.push(&crate::Route::NodeTasks {
                        nodename: node.to_string(),
                    });
                }
            })
            .into()
    }
}

impl Component for PveNodeDashboardPanel {
    type Message = Msg;
    type Properties = NodeDashboardPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: None,
            reload_timeout: None,
            load_guard: None,
            cmd_guard: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Load => {
                let link = ctx.link().clone();
                let url = get_status_url(&props.node);
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
                Ok(()) => {}
                Err(err) => {
                    let msg = format!("Command failed: {err}");
                    log::error!("{msg}");
                    ctx.link().show_snackbar(SnackBar::new().message(msg));
                }
            },
            Msg::Reboot => {
                self.node_command(ctx, "reboot");
            }
            Msg::Shutdown => {
                self.node_command(ctx, "shutdown");
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.data {
            Some(Ok(data)) => Column::new()
                .class(pwt::css::FlexFit)
                .padding(2)
                .gap(2)
                .with_child(self.view_status(ctx, data))
                .with_child(self.view_actions(ctx, data))
                .with_child(self.task_button(ctx))
                //.with_child(NodeResourcesPanel::new(props.node.clone(), props.vmid))
                .into(),
            Some(Err(err)) => pwt::widget::error_message(err).into(),
            None => Progress::new().class("pwt-delay-visibility").into(),
        }
    }
}

impl Into<VNode> for NodeDashboardPanel {
    fn into(self) -> VNode {
        let comp = VComp::new::<PveNodeDashboardPanel>(Rc::new(self), None);
        VNode::from(comp)
    }
}

use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Button, Card, Column, Container, Fa, List, Row};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::QemuConfig;

use crate::ListTile;

#[derive(Clone, PartialEq, Properties)]
pub struct VmConfigPanel {
    vmid: u64,
    node: AttrValue,
}

impl VmConfigPanel {
    pub fn new(node: impl Into<AttrValue>, vmid: u64) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

fn get_config_url(node: &str, vmid: u64) -> String {
    format!(
        "/nodes/{}/qemu/{}/config",
        percent_encode_component(node),
        vmid
    )
}

pub enum Msg {
    Load,
    LoadResult(Result<QemuConfig, Error>),
}

pub struct PveVmConfigPanel {
    data: Result<QemuConfig, String>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
}

fn create_config_tile(icon: &str, title: &str, subtitle: &str) -> ListTile {
    let icon: Html = Fa::new(icon).fixed_width().large_2x().into();

    ListTile::new()
        .class("pwt-d-flex")
        .class("pwt-gap-2")
        .class("pwt-scheme-surface")
        .border_top(true)
        .leading(icon)
        .title(title.to_string())
        .subtitle(subtitle.to_string())
}

impl PveVmConfigPanel {
    fn view_config(&self, ctx: &Context<Self>, data: &QemuConfig) -> Html {
        Card::new()
            .border(true)
            .padding(0)
            .class("pwt-flex-none pwt-overflow-hidden")
            .with_child(html! {<div class="pwt-p-2 pwt-font-size-title-large">{"Hardware"}</div>})
            .with_child(create_config_tile(
                "memory",
                data.memory.as_deref().unwrap_or("-"),
                "Memory",
            ))
            .with_child(create_config_tile(
                "cpu",
                data.cpu.as_deref().unwrap_or("-"),
                "Processor",
            ))
            .with_child(create_config_tile(
                "microchip",
                &data
                    .bios
                    .map(|b| b.to_string())
                    .unwrap_or(String::from("Default (SeaBIOS)")),
                "Bios",
            ))
            .into()
    }
}

impl Component for PveVmConfigPanel {
    type Message = Msg;
    type Properties = VmConfigPanel;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        Self {
            data: Err(format!("no data loaded")),
            reload_timeout: None,
            load_guard: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Load => {
                let link = ctx.link().clone();
                let url = get_config_url(&props.node, props.vmid);
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get(&url, None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.data = result.map_err(|err| err.to_string());
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.data {
            Ok(data) => self.view_config(ctx, data),
            Err(err) => Card::new()
                .border(true)
                .class("pwt-card-bordered")
                .with_child(pwt::widget::error_message(err))
                .into(),
        }
    }
}

impl From<VmConfigPanel> for VNode {
    fn from(props: VmConfigPanel) -> Self {
        let comp = VComp::new::<PveVmConfigPanel>(Rc::new(props), None);
        VNode::from(comp)
    }
}

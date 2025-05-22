use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Button, Card, Column, Container, Fa, ListTile, Row};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{IsRunning, QemuStatus};

use crate::{TopNavBar, VmConfigPanel};

#[derive(Clone, PartialEq, Properties)]
pub struct PageVmStatus {
    vmid: u64,
    node: AttrValue,
}

impl PageVmStatus {
    pub fn new(node: impl Into<AttrValue>, vmid: u64) -> Self {
        Self {
            node: node.into(),
            vmid,
        }
    }
}

pub struct PvePageVmStatus {
    data: Result<QemuStatus, String>,
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
}

pub enum Msg {
    Load,
    LoadResult(Result<QemuStatus, Error>),
    Start,
    Stop,
}

fn get_status_url(node: &str, vmid: u64, cmd: &str) -> String {
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
    fn view_status(&self, ctx: &Context<Self>, data: &QemuStatus) -> Html {
        let props = ctx.props();

        let vm_icon = large_fa_icon("desktop", data.status == IsRunning::Running);

        Card::new()
            .border(true)
            .class("pwt-d-flex pwt-gap-2")
            .class("pwt-align-items-center")
            .with_child(vm_icon)
            .with_child(
                Column::new()
                    .class("pwt-flex-fill")
                    .gap(1)
                    .with_child(html! {
                        <div class="pwt-font-size-title-medium">{
                            format!("{} {}", data.vmid, data.name.as_deref().unwrap_or(""))

                        }</div>
                    })
                    .with_child(html! {
                        <div class="pwt-font-size-title-small">{&props.node}</div>
                    }),
            )
            .with_child(html! {
                <div class="pwt-font-size-title-small">{data.status.to_string()}</div>
            })
            .into()
    }

    fn view_actions(&self, ctx: &Context<Self>, data: &QemuStatus) -> Html {
        let props = ctx.props();

        let running = data.status == IsRunning::Running;

        Row::new()
            .gap(2)
            .class(pwt::css::JustifyContent::SpaceBetween)
            .with_child(Button::new("Start").disabled(running))
            .with_child(Button::new("Stop").disabled(!running))
            .with_child(Button::new("Console").disabled(!running))
            .into()
    }
}

impl Component for PvePageVmStatus {
    type Message = Msg;
    type Properties = PageVmStatus;

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
                let url = get_status_url(&props.node, props.vmid, "current");
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
            Msg::Start => {}
            Msg::Stop => {}
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content: Html = match &self.data {
            Ok(data) => Column::new()
                .padding(2)
                .gap(2)
                .with_child(self.view_status(ctx, data))
                .with_child(self.view_actions(ctx, data))
                .with_child(VmConfigPanel::new(props.node.clone(), props.vmid))
                .into(),
            Err(err) => pwt::widget::error_message(err).into(),
        };

        Column::new()
            .class("pwt-fit")
            .with_child(
                TopNavBar::new()
                    .title(format!("VM {}", props.vmid))
                    .back("/resources"),
            )
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

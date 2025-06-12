use std::rc::Rc;

use anyhow::Error;
use gloo_timers::callback::Timeout;
use proxmox_yew_comp::http_post;
use pwt::widget::menu::{Menu, MenuItem, SplitButton};
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{Button, Card, Column, Fa, MiniScroll, MiniScrollMode, Row};
use pwt::AsyncPool;

use proxmox_yew_comp::{http_get, percent_encoding::percent_encode_component};

use pve_api_types::{IsRunning, QemuStatus};

use crate::widgets::{TopNavBar, VmConfigPanel};

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
    async_pool: AsyncPool,
}

pub enum Msg {
    Load,
    LoadResult(Result<QemuStatus, Error>),
    CommandResult(Result<String, Error>),
    Start,
    Stop,
    Shutdown,
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
            .class("pwt-elevation0")
            .class(pwt::css::JustifyContent::Center)
            .with_child("Task List")
            .onclick({
                let props = ctx.props();
                let url = format!(
                    "/resources/qemu/{}/{}/tasks",
                    percent_encode_component(&props.node),
                    props.vmid,
                );
                move |_| crate::goto_location(&url)
            })
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
            async_pool: AsyncPool::new(),
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
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content: Html = match &self.data {
            Ok(data) => Column::new()
                .class(pwt::css::FlexFit)
                .padding(2)
                .gap(2)
                .with_child(self.view_status(ctx, data))
                .with_child(self.view_actions(ctx, data))
                .with_child(self.task_button(ctx))
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

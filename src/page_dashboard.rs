use std::rc::Rc;

use anyhow::Error;
use js_sys::Date;
use wasm_bindgen::JsValue;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::touch::Fab;
use pwt::widget::form::{Field, Form, FormContext};
use pwt::widget::{Button, Card, Column, Container, MiniScroll, Panel, Progress, Row};

use proxmox_client::api_types::ClusterNodeIndexResponse;
use proxmox_yew_comp::http_get;

use crate::{Route, TopNavBar};

#[derive(Clone, PartialEq, Properties)]
pub struct PageDashboard {}

impl PageDashboard {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct PvePageDashboard {
    nodes: Result<Vec<ClusterNodeIndexResponse>, String>,
}

pub enum Msg {
    NodeLoadResult(Result<Vec<ClusterNodeIndexResponse>, Error>),
}

impl PvePageDashboard {
    fn load(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = http_get("/nodes", None).await;
            link.send_message(Msg::NodeLoadResult(result));
        });
    }

    fn create_tab_bar(&self, ctx: &Context<Self>) -> Html {
        let content = Row::new()
            .padding_y(1)
            .gap(2)
            .with_child(Button::new("Subscription"))
            .with_child(Button::new("Virtual Machines").icon_class("fa fa-desktop"))
            .with_child(Button::new("Containers").icon_class("fa fa-cube"));

        MiniScroll::new(content).into()
    }

    fn create_analytics_card(&self, ctx: &Context<Self>) -> Html {
        let content = match &self.nodes {
            Ok(list) => {
                let mut cpu = 0.0;
                let mut maxcpu = 0;
                let mut mem = 0;
                let mut maxmem = 0;
                let node_count = list.len();

                for node in list {
                    if let (Some(node_cpu), Some(node_maxcpu)) = (node.cpu, node.maxcpu) {
                        cpu += node_cpu;
                        maxcpu += node_maxcpu;
                    }
                    if let (Some(node_mem), Some(node_maxmem)) = (node.mem, node.maxmem) {
                        mem += node_mem;
                        maxmem += node_maxmem;
                    }
                }

                let mem = (mem as f64) / (1024.0 * 1024.0 *1024.0);
                let maxmem = (maxmem as f64) / (1024.0 * 1024.0 *1024.0);

                let cpu_percentage = if maxcpu == 0 {
                    0.0
                } else {
                    (cpu as f32) / (maxcpu as f32)
                };

                let mem_percentage = if maxmem <= 0.0 {
                    0.0
                } else {
                    (mem as f32) / (maxmem as f32)
                };

                Column::new()
                    .with_child(
                        Row::new()
                            .padding(2)
                            .border_bottom(true)
                            .with_child(
                                Column::new()
                                    .class("pwt-flex-fill")
                                    .gap(1)
                                    .with_child(html!{<div class="pwt-font-size-title-medium">{"CPU"}</div>})
                                    .with_child(html!{<div class="pwt-font-size-title-small">{format!("Cores {} Nodes {}", maxcpu, node_count)}</div>})
                            )
                            .with_child(
                                Progress::new().attribute("style", "width: 100px;").value(cpu_percentage)
                            )
                    )
                    .with_child(
                        Row::new()
                        .padding(2)
                        .with_child(
                            Column::new()
                                .class("pwt-flex-fill")
                                .gap(1)
                                .with_child(html!{<div class="pwt-font-size-title-medium">{"Memory"}</div>})
                                .with_child(html!{<div class="pwt-font-size-title-small">{format!("{:.2} Gib of {:.2} Gib", mem, maxmem)}</div>})
                        )
                        .with_child(
                            Progress::new().attribute("style", "width: 100px;").value(mem_percentage)
                        )
                    )
                    .into()
            }
            Err(err) => pwt::widget::error_message(err, "pwt-p-2"),
        };
        Card::new()
            .padding(0)
            .with_child(
                Column::new()
                    .padding(2)
                    .gap(1)
                    .border_bottom(true)
                    .with_child(html!{
                        <div class="pwt-font-size-title-large">{"Analytics"}</div>
                    })
                    .with_child(html!{
                        <div class="pwt-font-size-title-small">{"Usage acress all online nodes."}</div>
                    })
            )
            .with_child(content)
           .into()
    }

    fn create_nodes_card(&self, ctx: &Context<Self>) -> Html {
        let list = match &self.nodes {
            Ok(list) => {
                html! {
                    <div class="pwt-p-2">{"Nodes..."}</div>
                }
            }
            Err(err) => pwt::widget::error_message(err, "pwt-p-2"),
        };

        Card::new()
            .padding(0)
            .with_child(html! {
                <div class="pwt-p-2 pwt-border-bottom">
                    <div class="pwt-font-size-title-large">{"Nodes"}</div>
                </div>
            })
            .with_child(list)
            .into()
    }

    fn create_guests_card(&self, ctx: &Context<Self>) -> Html {
        Card::new()
            .padding(0)
            .with_child(html! {
                <div class="pwt-p-2 pwt-border-bottom">
                    <div class="pwt-font-size-title-large">{"Guests"}</div>
                </div>
            })
            .with_child(html! {
                <div class="pwt-p-2">{"Guests..."}</div>
            })
            .into()
    }
}

impl Component for PvePageDashboard {
    type Message = Msg;
    type Properties = PageDashboard;

    fn create(ctx: &Context<Self>) -> Self {
        let me = Self {
            nodes: Err(format!("no data loaded")),
        };
        me.load(ctx);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NodeLoadResult(result) => {
                self.nodes = result.map_err(|err| err.to_string());
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = Column::new()
            .padding(2)
            .gap(2)
            .with_child(self.create_tab_bar(ctx))
            .with_child(self.create_analytics_card(ctx))
            .with_child(self.create_nodes_card(ctx))
            .with_child(self.create_guests_card(ctx));

        /*
        let fab = Container::new()
            .class("pwt-position-absolute")
            .class("pwt-right-2 pwt-bottom-2")
            .with_child(
                Fab::new("fa fa-calendar")
                    .class("pwt-scheme-primary")
                    //.on_click(ctx.link().callback(|_| Msg::ShowDialog))
            );
        */

        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new())
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageDashboard {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageDashboard>(Rc::new(self), None);
        VNode::from(comp)
    }
}

use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Error;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::widget::{AlertDialog, Button, Card, Column, MiniScroll, Progress, Row};

use proxmox_client::api_types::{ClusterNodeIndexResponse, ClusterNodeIndexResponseStatus};
use proxmox_client::api_types::{ClusterResources, ClusterResourcesType};

use proxmox_yew_comp::http_get;

use crate::TopNavBar;

static SUBSCRIPTION_CONFIRMED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, PartialEq, Properties)]
pub struct PageDashboard {}

impl PageDashboard {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct PvePageDashboard {
    nodes: Result<Vec<ClusterNodeIndexResponse>, String>,
    resources: Result<Vec<ClusterResources>, String>,
}
pub enum Msg {
    NodeLoadResult(Result<Vec<ClusterNodeIndexResponse>, Error>),
    ResourcesLoadResult(Result<Vec<ClusterResources>, Error>),
    ConfirmSubscription,
}

impl PvePageDashboard {
    fn load(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = http_get("/nodes", None).await;
            link.send_message(Msg::NodeLoadResult(result));
        });
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = http_get("/cluster/resources", None).await;
            link.send_message(Msg::ResourcesLoadResult(result));
        });
    }

    fn create_tab_bar(&self, _ctx: &Context<Self>) -> Html {
        let content = Row::new()
            .padding_y(1)
            .gap(2)
            .with_child(Button::new("Subscription"))
            .with_child(Button::new("Virtual Machines").icon_class("fa fa-desktop"))
            .with_child(Button::new("Containers").icon_class("fa fa-cube"));

        MiniScroll::new(content).into()
    }

    fn create_analytics_card(&self, _ctx: &Context<Self>) -> Html {
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

                let mem = (mem as f64) / (1024.0 * 1024.0 * 1024.0);
                let maxmem = (maxmem as f64) / (1024.0 * 1024.0 * 1024.0);

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

    fn create_node_list_item(&self, item: &ClusterNodeIndexResponse) -> Row {
        let icon = html! {<i class={
            classes!(
                "pwt-font-size-title-large",
                "fa",
                "fa-server",
                (item.status == ClusterNodeIndexResponseStatus::Online).then(|| "pwt-color-primary"),
            )
        }/>};

        Row::new()
            .gap(2)
            .padding(2)
            .border_top(true)
            .class("pwt-align-items-center")
            .with_child(icon)
            .with_child(
                Column::new()
                    .class("pwt-flex-fill")
                    .gap(1)
                    .with_child(html! {
                        <div class="pwt-font-size-title-medium">{&item.node}</div>
                    }), //.with_child(html! {
                        //    <div class="pwt-font-size-title-small">{item.node.as_deref().unwrap()}</div>
                        //}),
            )
            .with_child(html! {
                <div class="pwt-font-size-title-small">{item.status}</div>
            })
    }

    fn create_nodes_card(&self, _ctx: &Context<Self>) -> Html {
        let list = match &self.nodes {
            Ok(list) => list
                .iter()
                .map(|item| self.create_node_list_item(item))
                .collect(),
            Err(err) => pwt::widget::error_message(err, "pwt-p-2"),
        };

        Card::new()
            .padding(0)
            .with_child(html! {<div class="pwt-p-2 pwt-font-size-title-large">{"Nodes"}</div>})
            .with_child(list)
            .into()
    }

    fn create_guest_info_row(icon_class: &str, text: &str, value: usize, large: bool) -> Row {
        let icon_size = if large {
            "pwt-font-size-title-large"
        } else {
            "pwt-ps-2 pwt-font-size-title-medium"
        };
        let font_size = if large {
            "pwt-font-size-title-medium"
        } else {
            "pwt-font-size-title-small"
        };

        Row::new()
            .padding(2)
            .gap(2)
            .border_top(true)
            .class("pwt-align-items-center")
            .with_child(html! {
                <i class={classes!(icon_size,  "fa-fw", icon_class.to_string())}/>
            })
            .with_child(html! {
                <div class={classes!("pwt-flex-fill", font_size)}>{text}</div>
            })
            .with_child(html! {
                <div class={font_size}>{value.to_string()}</div>
            })
    }

    fn create_guests_card(&self, _ctx: &Context<Self>) -> Html {
        let content = match &self.resources {
            Ok(list) => {
                let mut vm_count = 0;
                let mut vm_online_count = 0;
                let mut ct_count = 0;
                let mut ct_online_count = 0;

                for item in list {

                    if item.ty == ClusterResourcesType::Qemu {
                        vm_count += 1;
                        if item.status.as_deref() == Some("running") {
                            vm_online_count += 1;
                        }
                    }
                    if item.ty == ClusterResourcesType::Lxc {
                        ct_count += 1;
                        if item.status.as_deref() == Some("running") {
                            ct_online_count += 1;
                        }
                    }
                }

                Column::new()
                    .with_child(Self::create_guest_info_row(
                        "fa fa-desktop",
                        "Virtual Machines",
                        vm_count,
                        true,
                    ))
                    .with_child(Self::create_guest_info_row(
                        "fa fa-play pwt-color-primary",
                        "Online",
                        vm_online_count,
                        false,
                    ))
                    .with_child(Self::create_guest_info_row(
                        "fa fa-stop",
                        "Offline",
                        vm_count - vm_online_count,
                        false,
                    ))
                    .with_child(Self::create_guest_info_row(
                        "fa fa-cube",
                        "LXC Container",
                        ct_count,
                        true,
                    ))
                    .with_child(Self::create_guest_info_row(
                        "fa fa-play pwt-color-primary",
                        "Online",
                        ct_online_count,
                        false,
                    ))
                    .with_child(Self::create_guest_info_row(
                        "fa fa-stop",
                        "Offline",
                        ct_count - ct_online_count,
                        false,
                    ))
                    .into()
            }
            Err(err) => pwt::widget::error_message(err, "pwt-p-2"),
        };

        Card::new()
            .padding(0)
            .with_child(html! {
                <div class="pwt-p-2 pwt-font-size-title-large">{"Guests"}</div>
            })
            .with_child(content)
            .into()
    }

    fn create_subscription_alert(&self, ctx: &Context<Self>) -> AlertDialog {
        let msg = "
            One or more nodes do not have a valid subscription.\n\n
            The Proxmox team works very hard to make sure you are running the best
            software and getting stable updates and security enhancements,
            as well as quick enterprise support.\n\n
            Please consider to buy a subscription.
        ";

        AlertDialog::new(msg)
            .title("Subscription")
            .on_close(ctx.link().callback(|_| Msg::ConfirmSubscription))
    }
}

impl Component for PvePageDashboard {
    type Message = Msg;
    type Properties = PageDashboard;

    fn create(ctx: &Context<Self>) -> Self {
        let me = Self {
            nodes: Err(format!("no data loaded")),
            resources: Err(format!("no data loaded")),
        };
        me.load(ctx);
        me
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ResourcesLoadResult(result) => {
                self.resources = result.map_err(|err| err.to_string());
            }
            Msg::NodeLoadResult(result) => {
                self.nodes = result.map_err(|err| err.to_string());

                if let Ok(nodes) = &self.nodes {
                    let supported = nodes.iter().fold(true, |mut acc, item| {
                        if item.level.as_deref().unwrap_or("").is_empty() {
                            acc = false;
                        }
                        acc
                    });
                    if !SUBSCRIPTION_CONFIRMED.load(Ordering::Relaxed) {
                        SUBSCRIPTION_CONFIRMED.store(supported, Ordering::Relaxed);
                    }
                }
            }
            Msg::ConfirmSubscription => {
                SUBSCRIPTION_CONFIRMED.store(true, Ordering::Relaxed);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = Column::new()
            .class("pwt-flex-fill")
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

        let alert = (!SUBSCRIPTION_CONFIRMED.load(Ordering::Relaxed))
            .then(|| self.create_subscription_alert(ctx));

        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new())
            .with_child(content)
            .with_optional_child(alert)
            .into()
    }
}

impl Into<VNode> for PageDashboard {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageDashboard>(Rc::new(self), None);
        VNode::from(comp)
    }
}

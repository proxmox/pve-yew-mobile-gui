use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Error;

use proxmox_human_byte::HumanByte;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::widget::{Card, Column, Fa, List, ListTile};

use pve_api_types::{
    ClusterNodeIndexResponse, ClusterNodeIndexResponseStatus, ClusterResource, ClusterResourceType,
};

use proxmox_yew_comp::{http_get, SubscriptionAlert};

use crate::pages::ResourceFilter;
use crate::widgets::{icon_list_tile, list_tile_usage, TopNavBar};

static SUBSCRIPTION_CONFIRMED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, PartialEq, Properties)]
pub struct PageDashboard {}

impl PageDashboard {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct PvePageDashboard {
    nodes_loading: bool,
    nodes: Result<Vec<ClusterNodeIndexResponse>, String>,
    resources_loading: bool,
    resources: Result<Vec<ClusterResource>, String>,
    subscription_error: Option<String>, // None == Ok
    show_subscription_alert: bool,
}
pub enum Msg {
    NodeLoadResult(Result<Vec<ClusterNodeIndexResponse>, Error>),
    ResourcesLoadResult(Result<Vec<ClusterResource>, Error>),
    ConfirmSubscription,
    ShowSubscriptionAlert,
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

    fn create_subscription_card(&self, ctx: &Context<Self>) -> Option<Html> {
        if self.subscription_error.is_some() {
            Some(
                Card::new()
                    .padding(2)
                    .class("pwt-d-flex")
                    .class("pwt-interactive")
                    .class(pwt::css::JustifyContent::Center)
                    .with_child(tr!("Subscription"))
                    .onclick(ctx.link().callback(|_| Msg::ShowSubscriptionAlert))
                    .into(),
            )
        } else {
            None
        }
    }

    fn create_analytics_card(&self, _ctx: &Context<Self>) -> Html {
        let mut storage_used_size = 0;
        let mut storage_total_size = 0;
        let mut storage_percentage = 0.0;

        if let Ok(list) = &self.resources {
            for item in list {
                if item.ty == ClusterResourceType::Storage {
                    storage_used_size += item.disk.unwrap_or(0);
                    storage_total_size += item.maxdisk.unwrap_or(0);
                }
            }

            storage_percentage = if storage_total_size == 0 {
                0.0
            } else {
                (storage_used_size as f32) / (storage_total_size as f32)
            };
        }

        let loading = self.resources_loading || self.nodes_loading;

        let mut node_count = 0;
        let content: Html = match &self.nodes {
            Ok(list) => {
                let mut cpu = 0.0;
                let mut maxcpu = 0;
                let mut mem = 0.0;
                let mut maxmem = 0.0;

                node_count = list.len();

                for node in list {
                    if let (Some(node_cpu), Some(node_maxcpu)) = (node.cpu, node.maxcpu) {
                        cpu += node_cpu;
                        maxcpu += node_maxcpu;
                    }
                    if let (Some(node_mem), Some(node_maxmem)) = (node.mem, node.maxmem) {
                        mem += node_mem as f64;
                        maxmem += node_maxmem as f64;
                    }
                }

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

                let mut tiles: Vec<ListTile> = Vec::new();

                tiles.push(
                    icon_list_tile(Fa::new("cpu"), tr!("CPU"), None::<&str>, None).with_child(
                        list_tile_usage(format!("{:.2}", cpu), maxcpu.to_string(), cpu_percentage),
                    ),
                );

                tiles.push(
                    icon_list_tile(Fa::new("memory"), tr!("Memory"), None::<&str>, None)
                        .with_child(list_tile_usage(
                            HumanByte::new_binary(mem).to_string(),
                            HumanByte::new_binary(maxmem).to_string(),
                            mem_percentage,
                        )),
                );

                tiles.push(
                    icon_list_tile(Fa::new("database"), tr!("Storage"), None::<&str>, None)
                        .with_child(list_tile_usage(
                            HumanByte::new_binary(storage_used_size as f64).to_string(),
                            HumanByte::new_binary(storage_total_size as f64).to_string(),
                            storage_percentage,
                        )),
                );

                List::new(tiles.len() as u64, move |pos| tiles[pos as usize].clone())
                    .grid_template_columns("auto 1fr")
                    .into()
            }
            Err(_) if loading => html! {},
            Err(err) => pwt::widget::error_message(err).padding(2).into(),
        };

        crate::widgets::standard_card(
            tr!("Analytics"),
            format!("Usage across all ({node_count}) online nodes."),
        )
        .with_optional_child(
            loading.then(|| pwt::widget::Progress::new().class("pwt-delay-visibility")),
        )
        .with_child(content)
        .into()
    }

    fn create_nodes_card(&self, ctx: &Context<Self>) -> Html {
        let content: Html = match self.nodes.as_ref() {
            Ok(nodes) => {
                let nodes: Vec<ClusterNodeIndexResponse> = nodes.clone();
                let navigator = ctx.link().navigator().clone().unwrap();
                List::new(nodes.len() as u64, move |pos| {
                    let navigator = navigator.clone();
                    let item = &nodes[pos as usize];
                    let nodename = item.node.clone();
                    let subtitle = match item.level.as_deref() {
                        Some("") | None => "no subscription",
                        Some(level) => level,
                    };

                    icon_list_tile(
                        Fa::new("server").class(
                            (item.status == ClusterNodeIndexResponseStatus::Online)
                                .then(|| "pwt-color-primary"),
                        ),
                        nodename.clone(),
                        subtitle.to_string(),
                        Some(item.status.to_string().into()),
                    )
                    .interactive(true)
                    .onclick(Callback::from(
                        move |event: web_sys::MouseEvent| {
                            event.stop_propagation();
                            navigator.push(&crate::Route::Node {
                                nodename: nodename.clone(),
                            });
                        },
                    ))
                })
                .grid_template_columns("auto 1fr auto")
                .into()
            }
            Err(_) if self.nodes_loading => html! {},
            Err(err) => pwt::widget::error_message(err).into(),
        };

        crate::widgets::standard_card(tr!("Nodes"), None::<&str>)
            .with_optional_child(
                self.nodes_loading
                    .then(|| pwt::widget::Progress::new().class("pwt-delay-visibility")),
            )
            .with_child(content)
            .class("pwt-interactive")
            .onclick(Callback::from({
                let navigator = ctx.link().navigator().clone().unwrap();

                move |_| {
                    let filter = ResourceFilter {
                        nodes: true,
                        ..Default::default()
                    };
                    navigator.push_with_state(&crate::Route::Resources, filter);
                }
            }))
            .into()
    }

    fn create_guests_card(&self, ctx: &Context<Self>) -> Html {
        let content: Html = match &self.resources {
            Ok(list) => {
                let mut vm_count = 0;
                let mut vm_online_count = 0;
                let mut ct_count = 0;
                let mut ct_online_count = 0;
                let mut storage_count = 0;
                let mut storage_online_count = 0;

                for item in list {
                    if item.ty == ClusterResourceType::Qemu {
                        vm_count += 1;
                        if item.status.as_deref() == Some("running") {
                            vm_online_count += 1;
                        }
                    }
                    if item.ty == ClusterResourceType::Lxc {
                        ct_count += 1;
                        if item.status.as_deref() == Some("running") {
                            ct_online_count += 1;
                        }
                    }
                    if item.ty == ClusterResourceType::Storage {
                        storage_count += 1;
                        if item.status.as_deref() == Some("available") {
                            storage_online_count += 1;
                        }
                    }
                }

                let mut tiles: Vec<ListTile> = Vec::new();

                tiles.push(
                    icon_list_tile(
                        Fa::new("desktop"),
                        tr!("Virtual Machines"),
                        format!("{vm_count} ({vm_online_count} online)"),
                        None,
                    )
                    .onclick({
                        let navigator = ctx.link().navigator().clone().unwrap();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            let filter = ResourceFilter {
                                qemu: true,
                                ..Default::default()
                            };
                            navigator.push_with_state(&crate::Route::Resources, filter);
                        }
                    })
                    .interactive(true),
                );

                tiles.push(
                    icon_list_tile(
                        Fa::new("cube"),
                        tr!("LXC Container"),
                        format!("{ct_count} ({ct_online_count} online)"),
                        None,
                    )
                    .onclick({
                        let navigator = ctx.link().navigator().clone().unwrap();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            let filter = ResourceFilter {
                                lxc: true,
                                ..Default::default()
                            };
                            navigator.push_with_state(&crate::Route::Resources, filter);
                        }
                    })
                    .interactive(true),
                );

                tiles.push(
                    icon_list_tile(
                        Fa::new("database"),
                        tr!("Storage"),
                        format!("{storage_count} ({storage_online_count} online)"),
                        None,
                    )
                    .onclick({
                        let navigator = ctx.link().navigator().clone().unwrap();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            let filter = ResourceFilter {
                                storage: true,
                                ..Default::default()
                            };
                            navigator.push_with_state(&crate::Route::Resources, filter);
                        }
                    })
                    .interactive(true),
                );
                List::new(tiles.len() as u64, move |pos| tiles[pos as usize].clone())
                    .grid_template_columns("auto 1fr auto")
                    .into()
            }
            Err(_) if self.resources_loading => html! {},
            Err(err) => pwt::widget::error_message(err).padding(2).into(),
        };

        crate::widgets::standard_card(tr!("Resources"), None::<&str>)
            .class("pwt-interactive")
            .with_optional_child(
                self.resources_loading
                    .then(|| pwt::widget::Progress::new().class("pwt-delay-visibility")),
            )
            .with_child(content)
            .onclick({
                let navigator = ctx.link().navigator().clone().unwrap();
                move |_| {
                    let filter = ResourceFilter {
                        lxc: true,
                        qemu: true,
                        ..Default::default()
                    };
                    navigator.push_with_state(&crate::Route::Resources, filter);
                }
            })
            .into()
    }
}

impl Component for PvePageDashboard {
    type Message = Msg;
    type Properties = PageDashboard;

    fn create(ctx: &Context<Self>) -> Self {
        let me = Self {
            nodes_loading: true,
            nodes: Err(tr!("no data loaded")),
            resources_loading: true,
            resources: Err(tr!("no data loaded")),
            show_subscription_alert: false,
            subscription_error: None, // assume ok by default
        };
        me.load(ctx);
        me
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ResourcesLoadResult(result) => {
                self.resources_loading = false;
                self.resources = result.map_err(|err| err.to_string());
            }
            Msg::NodeLoadResult(result) => {
                self.nodes_loading = false;
                self.nodes = result.map_err(|err| err.to_string());

                if let Ok(nodes) = &self.nodes {
                    let mut level = None;
                    let mut mixed = false;

                    for item in nodes.iter() {
                        if item.status == ClusterNodeIndexResponseStatus::Offline {
                            continue;
                        }
                        let node_level = item.level.as_deref().unwrap_or("");
                        if node_level.is_empty() {
                            // no subscription beats all, set it and break the loop
                            level = None;
                            mixed = false;
                            break;
                        }
                        if level.is_none() {
                            level = Some(node_level);
                        } else if level != Some(node_level) {
                            mixed = true;
                        }
                    }

                    let single_node = nodes.len() == 1;

                    if level.is_some() {
                        if mixed {
                            self.subscription_error = Some(String::from("notsame"));
                        } else {
                            self.subscription_error = None;
                        }
                    } else {
                        self.subscription_error = Some(String::from(if single_node {
                            "notfound"
                        } else {
                            "notall"
                        }));
                    }

                    if !SUBSCRIPTION_CONFIRMED.load(Ordering::Relaxed) {
                        SUBSCRIPTION_CONFIRMED
                            .store(self.subscription_error.is_none(), Ordering::Relaxed);
                    }
                }
            }
            Msg::ConfirmSubscription => {
                self.show_subscription_alert = false;
                SUBSCRIPTION_CONFIRMED.store(true, Ordering::Relaxed);
            }
            Msg::ShowSubscriptionAlert => {
                if self.subscription_error.is_some() {
                    self.show_subscription_alert = true;
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = Column::new()
            .class("pwt-flex-fill")
            .class("pwt-overflow-auto")
            .padding(2)
            .gap(2)
            .with_optional_child(self.create_subscription_card(ctx))
            .with_child(self.create_analytics_card(ctx))
            .with_child(self.create_nodes_card(ctx))
            .with_child(self.create_guests_card(ctx));

        /*
        let fab = Container::new()
            .class("pwt-position-absolute")
            .style("right", "var(--pwt-spacer-2)")
            .style("bottom", "var(--pwt-spacer-2)")
            .with_child(
                Fab::new("fa fa-calendar")
                    .class("pwt-scheme-primary")
                    //.on_click(ctx.link().callback(|_| Msg::ShowDialog))
            );
        */

        let mut alert = None;
        if let Some(status) = &self.subscription_error {
            if self.show_subscription_alert || !SUBSCRIPTION_CONFIRMED.load(Ordering::Relaxed) {
                alert = Some(
                    SubscriptionAlert::new(status.clone())
                        .on_close(ctx.link().callback(|_| Msg::ConfirmSubscription)),
                );
            }
        }

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

use std::cell::RefCell;
use std::rc::Rc;

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

use proxmox_yew_comp::layout::card::standard_card;
use proxmox_yew_comp::layout::list_tile::{icon_list_tile, list_tile_usage};
use proxmox_yew_comp::layout::render_loaded_data;
use proxmox_yew_comp::{http_get, SubscriptionAlert};

use crate::pages::ResourceFilter;
use crate::widgets::TopNavBar;

#[derive(Clone, PartialEq, Properties)]
pub struct PageDashboard {}

impl PageDashboard {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Default)]
struct CachedData {
    nodes: Option<Result<Vec<ClusterNodeIndexResponse>, Error>>,
    resources: Option<Result<Vec<ClusterResource>, Error>>,
    subscription_confirmed: bool,
    subscription_error: Option<String>, // None == Ok
}

thread_local! {
    static CACHE: RefCell<CachedData> = RefCell::new(CachedData::default());
}

pub struct PvePageDashboard {
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
        if CACHE.with_borrow(|cache| cache.subscription_error.is_some()) {
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
        let content = CACHE.with_borrow(|cache| {
            let data = match (&cache.nodes, &cache.resources) {
                (Some(Ok(nodes)), Some(Ok(resources))) => Some(Ok((nodes, resources))),
                (Some(Err(err)), _) | (_, Some(Err(err))) => Some(Err(err)),
                (None, _) | (_, None) => None,
            };
            render_loaded_data(&data, |(node_list, resource_list)| {
                let mut cpu = 0.0;
                let mut maxcpu = 0;
                let mut mem = 0.0;
                let mut maxmem = 0.0;

                for node in node_list.iter() {
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
                    icon_list_tile(Fa::new("cpu"), tr!("CPU"), (), ()).with_child(list_tile_usage(
                        format!("{:.2}", cpu),
                        maxcpu.to_string(),
                        cpu_percentage,
                    )),
                );

                tiles.push(
                    icon_list_tile(Fa::new("memory"), tr!("Memory"), (), ()).with_child(
                        list_tile_usage(
                            HumanByte::new_binary(mem).to_string(),
                            HumanByte::new_binary(maxmem).to_string(),
                            mem_percentage,
                        ),
                    ),
                );

                let mut storage_used_size = 0;
                let mut storage_total_size = 0;

                for item in resource_list.iter() {
                    if item.ty == ClusterResourceType::Storage {
                        storage_used_size += item.disk.unwrap_or(0);
                        storage_total_size += item.maxdisk.unwrap_or(0);
                    }
                }

                let storage_percentage = if storage_total_size == 0 {
                    0.0
                } else {
                    (storage_used_size as f32) / (storage_total_size as f32)
                };

                tiles.push(
                    icon_list_tile(Fa::new("database"), tr!("Storage"), (), ()).with_child(
                        list_tile_usage(
                            HumanByte::new_binary(storage_used_size as f64).to_string(),
                            HumanByte::new_binary(storage_total_size as f64).to_string(),
                            storage_percentage,
                        ),
                    ),
                );

                List::from_tiles(tiles)
                    .grid_template_columns("auto 1fr")
                    .into()
            })
        });

        let node_count = CACHE.with_borrow(|cache| match &cache.nodes {
            Some(Ok(list)) => list.len(),
            _ => 1,
        });

        standard_card(
            tr!("Analytics"),
            format!("Usage across all ({node_count}) online nodes."),
            (),
        )
        .with_child(content)
        .into()
    }

    fn create_nodes_card(&self, ctx: &Context<Self>) -> Html {
        let content: Html = CACHE.with_borrow(|cache| {
            render_loaded_data(&cache.nodes, |nodes| {
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
                        item.status.to_string(),
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
            })
        });

        standard_card(tr!("Nodes"), (), ())
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
        let content: Html = CACHE.with_borrow(|cache| {
            render_loaded_data(&cache.resources, |list| {
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
                        (),
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
                        (),
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
                        (),
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
            })
        });

        standard_card(tr!("Resources"), (), ())
            .class("pwt-interactive")
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
            show_subscription_alert: false,
        };
        me.load(ctx);
        me
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ResourcesLoadResult(result) => {
                CACHE.with_borrow_mut(|cache| cache.resources = Some(result));
            }
            Msg::NodeLoadResult(result) => {
                CACHE.with_borrow_mut(|cache| {
                    cache.nodes = Some(result);

                    if let Some(Ok(nodes)) = &cache.nodes {
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
                                cache.subscription_error = Some(String::from("notsame"));
                            } else {
                                cache.subscription_error = None;
                            }
                        } else {
                            cache.subscription_error = Some(String::from(if single_node {
                                "notfound"
                            } else {
                                "notall"
                            }));
                        }

                        if !cache.subscription_confirmed {
                            cache.subscription_confirmed = cache.subscription_error.is_none();
                        }
                    }
                })
            }
            Msg::ConfirmSubscription => {
                self.show_subscription_alert = false;
                CACHE.with_borrow_mut(|cache| {
                    cache.subscription_confirmed = true;
                });
            }
            Msg::ShowSubscriptionAlert => CACHE.with_borrow(|cache| {
                if cache.subscription_error.is_some() {
                    self.show_subscription_alert = true;
                }
            }),
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

        let alert = CACHE.with_borrow(|cache| {
            let mut alert = None;
            if let Some(status) = &cache.subscription_error {
                if self.show_subscription_alert || !cache.subscription_confirmed {
                    alert = Some(
                        SubscriptionAlert::new(status.clone())
                            .on_close(ctx.link().callback(|_| Msg::ConfirmSubscription)),
                    );
                }
            }
            alert
        });

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

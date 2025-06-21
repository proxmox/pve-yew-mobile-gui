use std::cmp::Ordering;
use std::rc::Rc;

use anyhow::Error;

use gloo_timers::callback::Timeout;
use proxmox_human_byte::HumanByte;
use yew::virtual_dom::{VComp, VNode};
use yew_router::scope_ext::RouterScopeExt;

use pwt::prelude::*;
use pwt::touch::SideDialog;
use pwt::widget::form::{Checkbox, Field};
use pwt::widget::{ActionIcon, Card, Column, Fa, List, ListTile, Panel, Row, Trigger};
use pwt::AsyncAbortGuard;

use proxmox_yew_comp::http_get;
use pve_api_types::{ClusterResource, ClusterResourceType};

use crate::widgets::{icon_list_tile, list_tile_usage};

#[derive(Clone, PartialEq, Properties)]
pub struct PageResources {
    /// Initial filter value
    pub default_filter: ResourceFilter,
}

impl PageResources {
    pub fn new() -> Self {
        Self {
            default_filter: ResourceFilter::default(),
        }
    }

    pub fn new_with_filter(filter: ResourceFilter) -> Self {
        Self {
            default_filter: filter,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ResourceFilter {
    pub name: String,
    pub storage: bool,
    pub qemu: bool,
    pub lxc: bool,
    pub nodes: bool,
}

fn filter_match(item: &ClusterResource, filter: &ResourceFilter) -> bool {
    let mut include = false;

    if filter.storage && item.ty == ClusterResourceType::Storage {
        include = true;
    }

    if filter.nodes && item.ty == ClusterResourceType::Node {
        include = true;
    }

    if filter.qemu && item.ty == ClusterResourceType::Qemu {
        include = true;
    }

    if filter.lxc && item.ty == ClusterResourceType::Lxc {
        include = true;
    }

    if !(filter.storage || filter.nodes || filter.lxc || filter.qemu) {
        include = true;
    }

    if !include {
        return false;
    }

    if !filter.name.is_empty() {
        if item.id.to_lowercase().contains(&filter.name) {
            return true;
        }
        if let Some(name) = &item.name {
            if name.contains(&filter.name) {
                return true;
            }
        }
        false
    } else {
        true
    }
}

pub struct PvePageResources {
    reload_timeout: Option<Timeout>,
    load_guard: Option<AsyncAbortGuard>,
    data: Result<Vec<ClusterResource>, String>,
    filter: ResourceFilter,
    show_filter_dialog: bool,
}

pub enum Msg {
    Load,
    LoadResult(Result<Vec<ClusterResource>, Error>),
    SetTextFilter(String),
    ClearTextFilter,
    ShowFilterDialog,
    CloseFilterDialog,
    FilterLxc(bool),
    FilterQemu(bool),
    FilterStorage(bool),
    FilterNodes(bool),
}

impl PvePageResources {
    fn create_node_list_item(&self, ctx: &Context<Self>, item: &ClusterResource) -> ListTile {
        let nodename = item.node.clone().unwrap();
        icon_list_tile(
            Fa::new("server")
                .class((item.status.as_deref() == Some("online")).then(|| "pwt-color-primary")),
            nodename.clone(),
            match item.level.as_deref() {
                Some("") | None => "no subscription",
                Some(level) => level,
            }
            .to_string(),
            item.status.clone().map(|s| s.to_html()),
        )
        .interactive(true)
        .onclick({
            let navigator = ctx.link().navigator().clone().unwrap();
            move |_| {
                navigator.push(&crate::Route::Node {
                    nodename: nodename.clone(),
                });
            }
        })
        .into()
    }

    fn create_vm_list_item(&self, icon: &str, item: &ClusterResource) -> ListTile {
        icon_list_tile(
            Fa::new(icon)
                .class((item.status.as_deref() == Some("running")).then(|| "pwt-color-primary")),
            format!(
                "{} {}",
                item.vmid.unwrap(),
                item.name.as_deref().unwrap_or("")
            ),
            item.node.clone(),
            item.status.clone().map(|s| s.to_html()),
        )
        .interactive(true)
    }

    fn create_qemu_list_item(&self, ctx: &Context<Self>, item: &ClusterResource) -> ListTile {
        let vmid = item.vmid.unwrap();
        let nodename = item.node.clone().unwrap();
        self.create_vm_list_item("desktop", item).onclick({
            let navigator = ctx.link().navigator().clone().unwrap();
            move |_| {
                navigator.push(&crate::Route::Qemu {
                    vmid,
                    nodename: nodename.clone(),
                });
            }
        })
    }

    fn create_lxc_list_item(&self, ctx: &Context<Self>, item: &ClusterResource) -> ListTile {
        let vmid = item.vmid.unwrap();
        let nodename = item.node.clone().unwrap();
        self.create_vm_list_item("cube", item).onclick({
            let navigator = ctx.link().navigator().clone().unwrap();
            move |_| {
                navigator.push(&crate::Route::Lxc {
                    vmid,
                    nodename: nodename.clone(),
                });
            }
        })
    }

    fn create_storage_list_item(&self, ctx: &Context<Self>, item: &ClusterResource) -> ListTile {
        let name = item.storage.clone().unwrap();

        let mut tile = icon_list_tile(
            Fa::new("database")
                .class((item.status.as_deref() == Some("available")).then(|| "pwt-color-primary")),
            format!("{} {}", item.storage.clone().unwrap(), name,),
            item.node.clone(),
            item.status.clone().map(|s| s.to_html()),
        )
        .onclick({
            let navigator = ctx.link().navigator().clone().unwrap();
            move |_| {
                navigator.push(&crate::Route::Storage { name: name.clone() });
            }
        })
        .interactive(true);

        if item.disk.is_some() && item.maxdisk.is_some() {
            let used = HumanByte::new_binary(item.disk.unwrap_or(0) as f64);
            let total = HumanByte::new_binary(item.maxdisk.unwrap_or(0) as f64);

            let percentage = (item.disk.unwrap_or(0) as f32) / (item.maxdisk.unwrap_or(1) as f32);

            tile.add_child(list_tile_usage(
                used.to_string(),
                total.to_string(),
                percentage,
            ));
        }

        tile
    }

    fn create_resource_list(&self, ctx: &Context<Self>, data: &[ClusterResource]) -> Html {
        let mut filter = self.filter.clone();
        filter.name = filter.name.to_lowercase();

        let children: Vec<ListTile> = data
            .iter()
            .filter_map(|item| {
                if !filter_match(item, &filter) {
                    return None;
                }
                match item.ty {
                    ClusterResourceType::Qemu => Some(self.create_qemu_list_item(ctx, item)),
                    ClusterResourceType::Lxc => Some(self.create_lxc_list_item(ctx, item)),
                    ClusterResourceType::Storage => Some(self.create_storage_list_item(ctx, item)),
                    ClusterResourceType::Node => Some(self.create_node_list_item(ctx, item)),
                    ClusterResourceType::Pool
                    | ClusterResourceType::Openvz
                    | ClusterResourceType::Sdn => {
                        /* ignore for now  */
                        None
                    }
                }
            })
            .collect();

        if children.is_empty() {
            Card::new()
                .class("pwt-shape-none pwt-card-flat")
                .class("pwt-scheme-neutral")
                .with_child("List is empty.")
                .into()
        } else {
            List::new(children.len() as u64, move |pos| {
                children[pos as usize].clone()
            })
            .grid_template_columns("auto 1fr auto")
            .class("pwt-fit")
            .into()
        }
    }

    fn create_filter_panel(&self, ctx: &Context<Self>) -> Html {
        let grid = Column::new()
            .padding(2)
            .gap(2)
            .with_child(html! { <div style="grid-column: 1/-1;">{"Type"}</div>})
            .with_child(
                Checkbox::new()
                    .checked(self.filter.qemu)
                    .box_label("Qemu")
                    .on_change(ctx.link().callback(Msg::FilterQemu)),
            )
            .with_child(
                Checkbox::new()
                    .checked(self.filter.lxc)
                    .box_label("Lxc")
                    .on_change(ctx.link().callback(Msg::FilterLxc)),
            )
            .with_child(
                Checkbox::new()
                    .checked(self.filter.nodes)
                    .box_label("Nodes")
                    .on_change(ctx.link().callback(Msg::FilterNodes)),
            )
            .with_child(
                Checkbox::new()
                    .checked(self.filter.storage)
                    .box_label("Storage")
                    .on_change(ctx.link().callback(Msg::FilterStorage)),
            );

        Panel::new()
            .title("Filter")
            .with_child(html! { <hr/>})
            .with_child(grid)
            .into()
    }

    fn create_top_bar(&self, ctx: &Context<Self>) -> Html {
        let mut search = Field::new()
            .value(self.filter.name.clone())
            .on_change(ctx.link().callback(|value| Msg::SetTextFilter(value)))
            .class(pwt::css::Flex::Fill);

        search.add_trigger(
            Trigger::new(if self.filter.name.is_empty() {
                ""
            } else {
                "fa fa-times"
            })
            .on_activate(ctx.link().callback(|_| Msg::ClearTextFilter)),
            true,
        );

        let filter_button = ActionIcon::new("fa fa-lg fa-filter")
            .class("pwt-scheme-surface")
            .on_activate(ctx.link().callback(|_| Msg::ShowFilterDialog));

        let filter_dialog = self.show_filter_dialog.then(|| {
            SideDialog::new()
                .direction(pwt::touch::SideDialogLocation::Right)
                .on_close(ctx.link().callback(|_| Msg::CloseFilterDialog))
                .with_child(self.create_filter_panel(ctx))
        });

        Row::new()
            .gap(1)
            .padding(1)
            .attribute("role", "banner")
            .attribute("aria-label", "Proxmox VE")
            .class("pwt-navbar")
            .class("pwt-bg-color-primary pwt-color-on-primary")
            .class("pwt-align-items-center")
            .with_child(search)
            .with_child(filter_button)
            .with_optional_child(filter_dialog)
            .into()
    }
}

fn type_ordering(ty: ClusterResourceType) -> usize {
    match ty {
        ClusterResourceType::Lxc => 1,
        ClusterResourceType::Openvz => 1,
        ClusterResourceType::Qemu => 2,
        ClusterResourceType::Storage => 3,
        ClusterResourceType::Node => 4,
        ClusterResourceType::Pool => 5,
        ClusterResourceType::Sdn => 6,
    }
}

impl Component for PvePageResources {
    type Message = Msg;
    type Properties = PageResources;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        ctx.link().send_message(Msg::Load);

        let mut filter = props.default_filter.clone();

        if let Some(location) = ctx.link().location() {
            if let Some(state) = location.state::<ResourceFilter>() {
                log::info!("GOT LOCATION STATE {:?}", state);
                filter = state.as_ref().clone();
            }
        }

        Self {
            data: Err(format!("no data loaded")),
            filter,
            show_filter_dialog: false,
            reload_timeout: None,
            load_guard: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Load => {
                let link = ctx.link().clone();
                self.load_guard = Some(AsyncAbortGuard::spawn(async move {
                    let result = http_get("/cluster/resources", None).await;
                    link.send_message(Msg::LoadResult(result));
                }));
            }
            Msg::LoadResult(result) => {
                self.data = result.map_err(|err| err.to_string());
                let _ = self.data.as_mut().map(|d| {
                    d.sort_by(|item, other| {
                        let order = type_ordering(item.ty).cmp(&type_ordering(other.ty));
                        if order != Ordering::Equal {
                            return order;
                        }

                        item.id.cmp(&other.id)
                    })
                });
                let link = ctx.link().clone();
                self.reload_timeout = Some(Timeout::new(3000, move || {
                    link.send_message(Msg::Load);
                }));
            }
            Msg::SetTextFilter(text) => {
                self.filter.name = text;
            }
            Msg::ClearTextFilter => {
                self.filter.name = String::new();
            }
            Msg::ShowFilterDialog => {
                self.show_filter_dialog = true;
            }
            Msg::CloseFilterDialog => {
                self.show_filter_dialog = false;
            }
            Msg::FilterLxc(value) => {
                self.filter.lxc = value;
            }
            Msg::FilterQemu(value) => {
                self.filter.qemu = value;
            }
            Msg::FilterStorage(value) => {
                self.filter.storage = value;
            }
            Msg::FilterNodes(value) => {
                self.filter.nodes = value;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = match &self.data {
            Err(err) => pwt::widget::error_message(err).padding(2).into(),
            Ok(data) => self.create_resource_list(ctx, &data),
        };

        Column::new()
            .class("pwt-fit")
            .with_child(self.create_top_bar(ctx))
            .with_child(content)
            .into()
    }
}

impl Into<VNode> for PageResources {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageResources>(Rc::new(self), None);
        VNode::from(comp)
    }
}

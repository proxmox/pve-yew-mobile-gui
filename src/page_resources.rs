use std::rc::Rc;

use anyhow::Error;

use pwt::widget::form::{Checkbox, Field};
use pwt::widget::menu::{Menu, MenuButton, MenuItem};
use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::{Fab, SideDialog};
use pwt::widget::{ActionIcon, Card, Column, Container, Panel, Progress, Row, Trigger};

use proxmox_yew_comp::http_get;
use pve_api_types::{ClusterResource, ClusterResourceType};

#[derive(Clone, PartialEq, Properties)]
pub struct PageResources {}

impl PageResources {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Default)]
struct ResourceFilter {
    name: String,
    storage: bool,
    qemu: bool,
    lxc: bool,
    nodes: bool,
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
    data: Result<Vec<ClusterResource>, String>,
    filter: ResourceFilter,
    show_filter_dialog: bool,
}

pub enum Msg {
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
    fn load(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = http_get("/cluster/resources", None).await;
            link.send_message(Msg::LoadResult(result));
        });
    }

    fn create_vm_list_item(&self, icon: &str, item: &ClusterResource) -> Card {
        let icon = html! {<i class={
            classes!(
                "pwt-font-size-title-large",
                icon.to_string(),
                (item.status.as_deref() == Some("running")).then(|| "pwt-color-primary"),
            )
        }/>};

        Card::new()
            .class("pwt-d-flex pwt-gap-2")
            .class("pwt-shape-none pwt-card-flat pwt-interactive")
            .class("pwt-scheme-neutral")
            .padding_x(2)
            .padding_y(1)
            .border_bottom(true)
            .class("pwt-align-items-center")
            .with_child(icon)
            .with_child(
                Column::new()
                    .class("pwt-flex-fill")
                    .gap(1)
                    .with_child(html! {
                        <div class="pwt-font-size-title-medium">{
                            format!("{} {}", item.vmid.unwrap(), item.name.as_deref().unwrap_or(""))

                        }</div>
                    })
                    .with_child(html! {
                        <div class="pwt-font-size-title-small">{item.node.as_deref().unwrap()}</div>
                    }),
            )
            .with_child(html! {
                <div class="pwt-font-size-title-small">{item.status.as_deref().unwrap_or("")}</div>
            })
    }

    fn create_qemu_list_item(&self, _ctx: &Context<Self>, item: &ClusterResource) -> Html {
        let icon = "fa fa-fw fa-desktop";
        let vmid = item.vmid.unwrap();
        self.create_vm_list_item(icon, item)
            .onclick(Callback::from(move |_| {
                super::goto_location(&format!("/resources/qemu/{vmid}"));
            }))
            .into()
    }

    fn create_lxc_list_item(&self, _ctx: &Context<Self>, item: &ClusterResource) -> Html {
        let icon = "fa fa-fw fa-cube";
        self.create_vm_list_item(icon, item).into()
    }

    fn create_storage_list_item(&self, _ctx: &Context<Self>, item: &ClusterResource) -> Html {
        let row1 = Row::new()
            .gap(2)
            .class("pwt-align-items-flex-end")
            .with_child(
                Column::new()
                    .class("pwt-flex-fill")
                    .gap(1)
                    .with_child(html! {
                        <div class="pwt-font-size-title-medium">{item.storage.as_deref().unwrap()}</div>
                    })
                    .with_child(html! {
                        <div class="pwt-font-size-title-small">{item.node.as_deref().unwrap()}</div>
                    }),
            )
            .with_child(html!{
                <div class="pwt-font-size-title-small">{item.status.as_deref().unwrap_or("")}</div>
            });

        let icon = html! {<i class={
            classes!(
                "pwt-font-size-title-large",
                "fa",
                "fa-database",
                (item.status.as_deref() == Some("available")).then(|| "pwt-color-primary"),
            )
        }/>};

        let used = format!(
            "{:.2} Gib",
            (item.disk.unwrap_or(0) as f64) / (1024.0 * 1024.0 * 1024.0)
        );
        let total = format!(
            "{:.2} Gib",
            (item.maxdisk.unwrap_or(0) as f64) / (1024.0 * 1024.0 * 1024.0)
        );

        let row2 = Row::new()
            .gap(2)
            .class("pwt-align-items-flex-end")
            .with_child(icon)
            .with_child(html! {
                <div class="pwt-font-size-title-medium pwt-flex-fill">{used}</div>
            })
            .with_child(html! {
                <div class="pwt-font-size-title-medium">{format!("Total: {}", total)}</div>
            });

        let progress = Progress::new()
            .value((item.disk.unwrap_or(0) as f32) / (item.maxdisk.unwrap_or(1) as f32));

        Card::new()
            .class("pwt-d-flex pwt-flex-direction-column pwt-gap-1")
            .class("pwt-shape-none pwt-card-flat pwt-interactive")
            .class("pwt-scheme-neutral")
            .padding_x(2)
            .padding_y(1)
            .border_bottom(true)
            .with_child(row1)
            .with_child(row2)
            .with_child(progress)
            .into()
    }

    fn create_resource_list(&self, ctx: &Context<Self>, data: &[ClusterResource]) -> Html {
        let mut filter = self.filter.clone();
        filter.name = filter.name.to_lowercase();

        let children: Vec<Html> = data
            .iter()
            .filter_map(|item| {
                if !filter_match(item, &filter) {
                    return None;
                }
                match item.ty {
                    ClusterResourceType::Qemu => Some(self.create_qemu_list_item(ctx, item)),
                    ClusterResourceType::Lxc => Some(self.create_lxc_list_item(ctx, item)),
                    ClusterResourceType::Storage => Some(self.create_storage_list_item(ctx, item)),
                    _ => None,
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
            Column::new().class("pwt-fit").children(children).into()
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
            )
            .with_child(Checkbox::new().box_label("Qemu"));

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

impl Component for PvePageResources {
    type Message = Msg;
    type Properties = PageResources;

    fn create(ctx: &Context<Self>) -> Self {
        let me = Self {
            data: Err(format!("no data loaded")),
            filter: ResourceFilter::default(),
            show_filter_dialog: false,
        };
        me.load(ctx);
        me
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadResult(result) => {
                self.data = result.map_err(|err| err.to_string());
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

        let fab = Container::new()
            .class("pwt-position-absolute")
            .style("right", "var(--pwt-spacer-2)")
            .style("bottom", "var(--pwt-spacer-2)")
            .with_child(
                Fab::new("fa fa-calendar").class("pwt-scheme-primary"), //.on_click(ctx.link().callback(|_| Msg::ShowDialog))
            );

        Column::new()
            .class("pwt-fit")
            .with_child(self.create_top_bar(ctx))
            .with_child(content)
            .with_child(fab)
            .into()
    }
}

impl Into<VNode> for PageResources {
    fn into(self) -> VNode {
        let comp = VComp::new::<PvePageResources>(Rc::new(self), None);
        VNode::from(comp)
    }
}

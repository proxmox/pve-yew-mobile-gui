use std::rc::Rc;

use anyhow::Error;

use yew::virtual_dom::{VComp, VNode};

use pwt::prelude::*;
use pwt::touch::Fab;
use pwt::widget::{Progress, Card, Column, Container, Row};

use proxmox_client::api_types::{ClusterResourceKind, ClusterResources, ClusterResourcesType};
use proxmox_yew_comp::http_get;

use crate::{Route, TopNavBar};

#[derive(Clone, PartialEq, Properties)]
pub struct PageResources {}

impl PageResources {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct PvePageResources {
    data: Result<Vec<ClusterResources>, String>,
}

pub enum Msg {
    LoadResult(Result<Vec<ClusterResources>, Error>),
}

impl PvePageResources {
    fn load(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = http_get("/cluster/resources", None).await;
            link.send_message(Msg::LoadResult(result));
        });
    }

    fn create_vm_list_item(&self, icon: &str, item: &ClusterResources) -> Card {
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

    fn create_qemu_list_item(&self, ctx: &Context<Self>, item: &ClusterResources) -> Html {
        let icon = "fa fa-fw fa-desktop";
        let vmid = item.vmid.unwrap();
        self.create_vm_list_item(icon, item)
            .onclick(Callback::from(move |_| {
                super::goto_location(&format!("/resources/qemu/{vmid}"));
            }))
            .into()
    }

    fn create_lxc_list_item(&self, ctx: &Context<Self>, item: &ClusterResources) -> Html {
        let icon = "fa fa-fw fa-cube";
        self.create_vm_list_item(icon, item).into()
    }

    fn create_storage_list_item(&self, ctx: &Context<Self>, item: &ClusterResources) -> Html {
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

        let used = format!("{:.2} Gib", (item.disk.unwrap_or(0) as f64) / (1024.0 * 1024.0 *1024.0));
        let total = format!("{:.2} Gib", (item.maxdisk.unwrap_or(0) as f64) / (1024.0 * 1024.0 *1024.0));

        let row2 = Row::new()
            .gap(2)
            .class("pwt-align-items-flex-end")
            .with_child(icon)
            .with_child(html!{
                <div class="pwt-font-size-title-medium pwt-flex-fill">{used}</div>
            })
            .with_child(html!{
                <div class="pwt-font-size-title-medium">{format!("Total: {}", total)}</div>
            });

        let progress = Progress::new()
            .value((item.disk.unwrap_or(0) as f32)/(item.maxdisk.unwrap_or(1) as f32));

        Card::new()
            .class("pwt-d-flex pwt-flex-column pwt-gap-1")
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

    fn create_resource_list(&self, ctx: &Context<Self>, data: &[ClusterResources]) -> Html {
        let children = data.iter().filter_map(|item| match item.ty {
            ClusterResourcesType::Qemu => Some(self.create_qemu_list_item(ctx, item)),
            ClusterResourcesType::Lxc => Some(self.create_lxc_list_item(ctx, item)),
            ClusterResourcesType::Storage => Some(self.create_storage_list_item(ctx, item)),
            _ => None,
        });
        Column::new().children(children).into()
    }
}

impl Component for PvePageResources {
    type Message = Msg;
    type Properties = PageResources;

    fn create(ctx: &Context<Self>) -> Self {
        let me = Self {
            data: Err(format!("no data loaded")),
        };
        me.load(ctx);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadResult(result) => {
                self.data = result.map_err(|err| err.to_string());
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let content = match &self.data {
            Err(err) => pwt::widget::error_message(err, "pwt-p-2"),
            Ok(data) => self.create_resource_list(ctx, &data),
        };

        let fab = Container::new()
            .class("pwt-position-absolute")
            .class("pwt-right-2 pwt-bottom-2")
            .with_child(
                Fab::new("fa fa-calendar")
                    .class("pwt-scheme-primary")
                    //.on_click(ctx.link().callback(|_| Msg::ShowDialog))
            );

        Column::new()
            .class("pwt-fit")
            .with_child(TopNavBar::new())
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

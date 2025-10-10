use std::rc::Rc;

use anyhow::Error;
use proxmox_yew_comp::http_get;
use serde_json::json;

use pve_api_types::{ClusterResource, ClusterResourceKind, ClusterResourceType};

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::Key;

use pwt::prelude::*;

use pwt::props::{FieldBuilder, LoadCallback, WidgetBuilder, WidgetStyleBuilder};
use pwt::state::Store;
use pwt::widget::data_table::{DataTable, DataTableColumn, DataTableHeader};
use pwt::widget::form::{Selector, SelectorRenderArgs};
use pwt::widget::{Fa, GridPicker, Row};

use pwt_macros::{builder, widget};

use crate::widgets::title_subtitle_column;

#[derive(PartialEq, Clone, Copy)]
pub enum PveGuestType {
    Qemu,
    Lxc,
}

#[widget(comp=PveGuestSelectorComp, @input)]
#[derive(Clone, Properties, PartialEq)]
#[builder]
pub struct PveGuestSelector {
    /// The default value
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<AttrValue>,

    /// Change callback
    #[builder_cb(IntoEventCallback, into_event_callback, Option<ClusterResource>)]
    #[prop_or_default]
    pub on_change: Option<Callback<Option<ClusterResource>>>,

    /// The guest type to show (any by default)
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub guest_type: Option<PveGuestType>,

    /// Include templates
    ///
    /// Some(false): do not include templates
    /// Some(ture): olny list templates
    /// None: include templates
    #[builder]
    #[prop_or(Some(false))]
    pub templates: Option<bool>,

    /// If set, automatically selects the first value from the store (if no default is selected)
    #[builder]
    #[prop_or(false)]
    pub autoselect: bool,

    /// Layout for mobile devices.
    #[builder]
    #[prop_or(false)]
    pub mobile: bool,
}

impl PveGuestSelector {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct PveGuestSelectorComp {
    store: Store<ClusterResource>,
    load_callback: LoadCallback<Vec<ClusterResource>>,
}

impl PveGuestSelectorComp {
    async fn get_guest_list(
        guest_type: Option<PveGuestType>,
        templates: Option<bool>,
    ) -> Result<Vec<ClusterResource>, Error> {
        let url = format!("/cluster/resources");
        let param = json!({ "type": ClusterResourceKind::Vm });

        let mut guest_list: Vec<ClusterResource> = http_get(url, Some(param)).await?;

        if let Some(guest_type) = guest_type {
            let resource_type = match guest_type {
                PveGuestType::Qemu => ClusterResourceType::Qemu,
                PveGuestType::Lxc => ClusterResourceType::Lxc,
            };
            guest_list = guest_list
                .into_iter()
                .filter(move |item| item.ty == resource_type && item.vmid.is_some())
                .filter(move |item| match templates {
                    None::<_> => true,
                    Some(false) => item.template != Some(true),
                    Some(true) => item.template == Some(true),
                })
                .collect();
        }

        guest_list.sort_by(|a, b| a.vmid.cmp(&b.vmid));
        Ok(guest_list)
    }

    fn create_load_callback(ctx: &yew::Context<Self>) -> LoadCallback<Vec<ClusterResource>> {
        let props = ctx.props();
        let guest_type = props.guest_type;
        let templates = props.templates.clone();
        (move || Self::get_guest_list(guest_type, templates)).into()
    }
}

impl Component for PveGuestSelectorComp {
    type Message = ();
    type Properties = PveGuestSelector;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {
            store: Store::with_extract_key(|item: &ClusterResource| Key::from(item.vmid.unwrap())),
            load_callback: Self::create_load_callback(ctx),
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if old_props.guest_type != props.guest_type {
            self.load_callback = Self::create_load_callback(ctx);
        }

        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();

        let on_change = {
            let on_change = props.on_change.clone();
            let store = self.store.clone();
            move |key: Key| {
                if let Some(on_change) = &on_change {
                    let result = store
                        .read()
                        .iter()
                        .find(|e| key == store.extract_key(e))
                        .map(|e| e.clone());
                    on_change.emit(result);
                }
            }
        };

        let mobile = props.mobile;

        Selector::new(
            self.store.clone(),
            move |args: &SelectorRenderArgs<Store<ClusterResource>>| {
                GridPicker::new(
                    DataTable::new(
                        if mobile { columns_mobile() } else { columns() },
                        args.store.clone(),
                    )
                    .min_width(300)
                    .show_header(!mobile)
                    .header_focusable(false)
                    .class(pwt::css::FlexFit),
                )
                .selection(args.selection.clone())
                .on_select(args.controller.on_select_callback())
                .into()
            },
        )
        .loader(self.load_callback.clone())
        .with_std_props(&props.std_props)
        .with_input_props(&props.input_props)
        .autoselect(props.autoselect)
        .on_change(on_change)
        .default(props.default.clone())
        .into()
    }
}

fn columns_mobile() -> Rc<Vec<DataTableHeader<ClusterResource>>> {
    Rc::new(vec![DataTableColumn::new(tr!("Id"))
        .get_property(|entry: &ClusterResource| entry.vmid.as_ref().unwrap())
        .sort_order(true)
        .render(|item: &ClusterResource| {
            let icon = match item.ty {
                ClusterResourceType::Qemu => "desktop",
                ClusterResourceType::Lxc => "cube",
                _ => "question",
            };
            let title = format!(
                "{} {}",
                item.vmid.unwrap(),
                item.name.as_deref().unwrap_or("")
            );
            Row::new()
                .class(pwt::css::AlignItems::Center)
                .gap(2)
                .with_child(Fa::new(icon).large_2x().class(
                    (item.status.as_deref() == Some("running")).then(|| "pwt-color-primary"),
                ))
                .with_child(title_subtitle_column(title, item.node.clone()))
                .with_flex_spacer()
                .with_child(item.status.as_deref().unwrap_or("-"))
                .into()
        })
        .into()])
}

fn columns() -> Rc<Vec<DataTableHeader<ClusterResource>>> {
    Rc::new(vec![
        DataTableColumn::new(tr!("Id"))
            .width("10em")
            .get_property(|entry: &ClusterResource| entry.vmid.as_ref().unwrap())
            .sort_order(true)
            .into(),
        DataTableColumn::new(tr!("Name"))
            .flex(1)
            .get_property(|entry: &ClusterResource| entry.name.as_deref().unwrap_or("-"))
            .sort_order(true)
            .into(),
        DataTableColumn::new(tr!("Node"))
            .get_property(|entry: &ClusterResource| entry.node.as_deref().unwrap_or("-"))
            .sort_order(true)
            .into(),
        DataTableColumn::new(tr!("Status"))
            .get_property(|entry: &ClusterResource| entry.status.as_deref().unwrap_or("-"))
            .sort_order(true)
            .into(),
        DataTableColumn::new(tr!("Type"))
            .width("10em")
            .render(|entry: &ClusterResource| {
                match entry.ty {
                    ClusterResourceType::Qemu => tr!("Virtual Machine"),
                    ClusterResourceType::Lxc => tr!("LXC Container"),
                    other => other.to_string(),
                }
                .into()
            })
            .into(),
    ])
}

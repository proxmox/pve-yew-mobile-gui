use std::rc::Rc;

use anyhow::Error;
use pwt::widget::GridPicker;
use serde_json::json;

use proxmox_human_byte::HumanByte;
use pve_api_types::StorageContent;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::Key;

use pwt::prelude::*;

use pwt::props::{FieldBuilder, LoadCallback, WidgetBuilder, WidgetStyleBuilder};
use pwt::state::Store;
use pwt::widget::data_table::{DataTable, DataTableColumn, DataTableHeader};
use pwt::widget::form::{Selector, SelectorRenderArgs};

use pwt_macros::{builder, widget};

use proxmox_yew_comp::http_get;
use proxmox_yew_comp::percent_encoding::percent_encode_component;

use crate::StorageEntry;

#[widget(comp=PveStorageContentSelectorComp, @input)]
#[derive(Clone, Properties, PartialEq)]
#[builder]
pub struct PveStorageContentSelector {
    /// The default value
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<AttrValue>,

    /// Change callback
    #[builder_cb(IntoEventCallback, into_event_callback, Option<AttrValue>)]
    #[prop_or_default]
    pub on_change: Option<Callback<Option<AttrValue>>>,

    /// The node to query
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub node: Option<AttrValue>,

    /// The storage to query
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub storage: Option<AttrValue>,

    /// Only list content of this type.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub content_filter: Option<StorageContent>,

    /// Only list images for this VM
    #[prop_or_default]
    vmid_filter: Option<u32>,

    /// If set, automatically selects the first value from the store (if no default is selected)
    #[builder]
    #[prop_or(false)]
    pub autoselect: bool,
}

impl PveStorageContentSelector {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct PveStorageContentSelectorComp {
    store: Store<StorageEntry>,
    load_callback: LoadCallback<Vec<StorageEntry>>,
    //validate_fn: pwt::widget::form::ValidateFn<(String, Store<StorageEntry>)>,
}

impl PveStorageContentSelectorComp {
    async fn get_storage_content(
        node: AttrValue,
        storage: Option<AttrValue>,
        content_filter: Option<StorageContent>,
        vmid_filter: Option<u32>,
    ) -> Result<Vec<StorageEntry>, Error> {
        let storage = match storage {
            Some(storage) => storage,
            None::<_> => return Ok(Vec::new()),
        };
        // fixme: Howto use PveClient trait (Send,Sync problem)?
        let url = format!(
            "/nodes/{}/storage/{}/content",
            percent_encode_component(&*node),
            percent_encode_component(&storage)
        );
        let mut param = json!({});
        if let Some(content) = content_filter {
            param["content"] = content.to_string().into();
        }
        if let Some(vmid) = vmid_filter {
            param["vmid"] = vmid.into();
        }

        let content: Vec<StorageEntry> = http_get(url, Some(param)).await?;

        Ok(content)
    }

    fn create_load_callback(ctx: &yew::Context<Self>) -> LoadCallback<Vec<StorageEntry>> {
        let props = ctx.props();
        let node = props.node.clone();
        let storage = props.storage.clone();
        let content_filter = props.content_filter.clone();
        let vmid_filter = props.vmid_filter;

        (move || {
            Self::get_storage_content(
                node.clone().unwrap_or("localhost".into()),
                storage.clone(),
                content_filter.clone(),
                vmid_filter.clone(),
            )
        })
        .into()
    }
}

impl Component for PveStorageContentSelectorComp {
    type Message = ();
    type Properties = PveStorageContentSelector;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            store: Store::with_extract_key(|storage: &StorageEntry| {
                Key::from(storage.volid.as_str())
            }),
            load_callback: Self::create_load_callback(ctx),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.load_callback = Self::create_load_callback(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        Selector::new(
            self.store.clone(),
            move |args: &SelectorRenderArgs<Store<StorageEntry>>| {
                GridPicker::new(
                    DataTable::new(columns(), args.store.clone())
                        .min_width(300)
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
        //.validate(self.validate_fn.clone())
        //.on_change(on_change)
        .default(props.default.clone())
        .into()
    }
}

fn columns() -> Rc<Vec<DataTableHeader<StorageEntry>>> {
    Rc::new(vec![
        DataTableColumn::new(tr!("Name"))
            .get_property(|entry: &StorageEntry| match entry.volid.split_once('/') {
                None => entry.volid.as_str(),
                Some((_, name)) => name,
            })
            .sort_order(true)
            .into(),
        DataTableColumn::new(tr!("Format"))
            .get_property(|entry: &StorageEntry| &entry.format)
            .into(),
        DataTableColumn::new(tr!("Size"))
            .sorter(|a: &StorageEntry, b: &StorageEntry| a.size.cmp(&b.size))
            .render(|entry: &StorageEntry| HumanByte::new_decimal(entry.size as f64).into())
            .into(),
    ])
}

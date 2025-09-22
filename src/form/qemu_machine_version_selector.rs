use std::rc::Rc;

use anyhow::{format_err, Error};
use proxmox_yew_comp::http_get;
use pwt::widget::GridPicker;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::Key;

use pwt::prelude::*;

use pwt::props::{FieldBuilder, LoadCallback, WidgetBuilder, WidgetStyleBuilder};
use pwt::state::Store;
use pwt::widget::data_table::{DataTable, DataTableColumn, DataTableHeader};
use pwt::widget::form::{Selector, SelectorRenderArgs, ValidateFn};

use pwt_macros::{builder, widget};

use crate::api_types::{QemuMachineInfo, QemuMachineType};

#[widget(comp=QemuMachineVersionSelectorComp, @input)]
#[derive(Clone, Properties, PartialEq)]
#[builder]
pub struct QemuMachineVersionSelector {
    /// The default value
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<AttrValue>,

    /// Change callback
    #[builder_cb(IntoEventCallback, into_event_callback, Option<AttrValue>)]
    #[prop_or_default]
    pub on_change: Option<Callback<Option<AttrValue>>>,

    /// List versions for this machine type.
    pub machine_type: QemuMachineType,

    /// If set, automatically selects the first value from the store (if no default is selected)
    #[builder]
    #[prop_or(false)]
    pub autoselect: bool,
}

impl QemuMachineVersionSelector {
    pub fn new(machine_type: QemuMachineType) -> Self {
        yew::props!(Self { machine_type })
    }
}

pub struct QemuMachineVersionSelectorComp {
    store: Store<QemuMachineInfo>,
    load_callback: LoadCallback<Vec<QemuMachineInfo>>,
    validate_fn: pwt::widget::form::ValidateFn<(String, Store<QemuMachineInfo>)>,
}

// We cannot use store.set_filter(), because GridPicker overwrites that
async fn get_machine_list(machine_type: QemuMachineType) -> Result<Vec<QemuMachineInfo>, Error> {
    let url = "/nodes/localhost/capabilities/qemu/machines";
    let model_list: Vec<QemuMachineInfo> = http_get(url, None).await?;
    let model_list: Vec<QemuMachineInfo> = model_list
        .into_iter()
        .filter(|item| item.ty == machine_type)
        .collect();

    Ok(model_list)
}

impl Component for QemuMachineVersionSelectorComp {
    type Message = ();
    type Properties = QemuMachineVersionSelector;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let props = ctx.props();
        let validate_fn = ValidateFn::new({
            let machine_type = props.machine_type;
            move |(value, store): &(String, Store<QemuMachineInfo>)| {
                store
                    .read()
                    .iter()
                    .find(|item| item.ty == machine_type && &item.id == value)
                    .ok_or_else(|| format_err!("no such item"))
                    .map(|_| ())
            }
        });

        let store = Store::with_extract_key(|info: &QemuMachineInfo| Key::from(info.id.as_str()));

        Self {
            store,
            load_callback: LoadCallback::new({
                let machine_type = props.machine_type;
                move || get_machine_list(machine_type)
            }),
            validate_fn,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.machine_type != old_props.machine_type {
            self.load_callback = LoadCallback::new({
                let machine_type = props.machine_type;
                move || get_machine_list(machine_type)
            });
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
                        .map(|e| e.id.clone().into());
                    on_change.emit(result);
                }
            }
        };

        Selector::new(
            self.store.clone(),
            move |args: &SelectorRenderArgs<Store<QemuMachineInfo>>| {
                GridPicker::new(
                    DataTable::new(columns(), args.store.clone())
                        .min_width(200)
                        .show_header(false)
                        .header_focusable(false)
                        .class(pwt::css::FlexFit),
                )
                .show_filter(false)
                .selection(args.selection.clone())
                .on_select(args.controller.on_select_callback())
                .into()
            },
        )
        .loader(self.load_callback.clone())
        .with_std_props(&props.std_props)
        .with_input_props(&props.input_props)
        .autoselect(props.autoselect)
        .validate(self.validate_fn.clone())
        .on_change(on_change)
        .default(props.default.clone())
        .placeholder(tr!("Latest"))
        .render_value(|id: &AttrValue| extract_version_text(id.as_str()).into())
        .into()
    }
}

fn extract_version_text(id: &str) -> String {
    if id == "pc" || id == "q35" {
        return tr!("Latest");
    }
    if id.starts_with("pc-q35-") {
        return id[7..].to_string();
    }
    if id.starts_with("pc-i440fx-") {
        return id[10..].to_string();
    }
    if id.starts_with("pc-") {
        return id[3..].to_string();
    }
    if id.starts_with("virt-") {
        return id[5..].to_string();
    }
    id.to_string()
}

fn columns() -> Rc<Vec<DataTableHeader<QemuMachineInfo>>> {
    Rc::new(vec![DataTableColumn::new(tr!("Version"))
        .get_property(|entry: &QemuMachineInfo| &entry.version)
        .render(|entry: &QemuMachineInfo| entry.version.clone().into())
        .into()])
}

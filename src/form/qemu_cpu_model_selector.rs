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

use crate::widgets::title_subtitle_column;
use crate::QemuCpuModel;

#[widget(comp=QemuCpuModelSelectorComp, @input)]
#[derive(Clone, Properties, PartialEq)]
#[builder]
pub struct QemuCpuModelSelector {
    /// The default value
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<AttrValue>,

    /// Change callback
    #[builder_cb(IntoEventCallback, into_event_callback, Option<AttrValue>)]
    #[prop_or_default]
    pub on_change: Option<Callback<Option<AttrValue>>>,

    /// If set, automatically selects the first value from the store (if no default is selected)
    #[builder]
    #[prop_or(false)]
    pub autoselect: bool,

    /// Layout for mobile devices.
    #[builder]
    #[prop_or(false)]
    pub mobile: bool,
}

impl QemuCpuModelSelector {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct QemuCpuModelSelectorComp {
    store: Store<QemuCpuModel>,
    load_callback: LoadCallback<Vec<QemuCpuModel>>,
    validate_fn: pwt::widget::form::ValidateFn<(String, Store<QemuCpuModel>)>,
}

async fn get_cpu_model_list() -> Result<Vec<QemuCpuModel>, Error> {
    let url = "/nodes/localhost/capabilities/qemu/cpu";
    let mut model_list: Vec<QemuCpuModel> = http_get(url, None).await?;
    model_list.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(model_list)
}

impl Component for QemuCpuModelSelectorComp {
    type Message = ();
    type Properties = QemuCpuModelSelector;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        let validate_fn = ValidateFn::new(|(value, store): &(String, Store<QemuCpuModel>)| {
            store
                .read()
                .iter()
                .find(|item| item.name == *value)
                .ok_or_else(|| format_err!("no such item"))
                .map(|_| ())
        });
        Self {
            store: Store::with_extract_key(|info: &QemuCpuModel| Key::from(info.name.as_str())),
            load_callback: LoadCallback::new(get_cpu_model_list),
            validate_fn,
        }
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
                        .map(|e| e.name.clone().into());
                    on_change.emit(result);
                }
            }
        };

        let mobile = props.mobile;

        Selector::new(
            self.store.clone(),
            move |args: &SelectorRenderArgs<Store<QemuCpuModel>>| {
                GridPicker::new(
                    DataTable::new(
                        if mobile { columns_mobile() } else { columns() },
                        args.store.clone(),
                    )
                    .min_width(200)
                    .show_header(!mobile)
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
        .placeholder("kvm64")
        .into()
    }
}

fn columns_mobile() -> Rc<Vec<DataTableHeader<QemuCpuModel>>> {
    Rc::new(vec![DataTableColumn::new(tr!("Name"))
        .get_property(|entry: &QemuCpuModel| &entry.name)
        //.sort_order(true)
        .render(|entry: &QemuCpuModel| {
            title_subtitle_column(entry.name.clone(), entry.vendor.clone()).into()
        })
        .into()])
}

fn columns() -> Rc<Vec<DataTableHeader<QemuCpuModel>>> {
    Rc::new(vec![
        DataTableColumn::new(tr!("Name"))
            .get_property(|entry: &QemuCpuModel| &entry.name)
            //.sort_order(true)
            .into(),
        DataTableColumn::new(tr!("Vendor"))
            .get_property(|entry: &QemuCpuModel| &entry.vendor)
            .into(),
    ])
}

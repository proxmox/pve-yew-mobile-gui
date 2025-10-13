use std::rc::Rc;

use pwt::state::Store;

use pwt::prelude::*;
use pwt::widget::data_table::{DataTable, DataTableColumn, DataTableHeader};
use pwt::widget::form::{Number, Selector, SelectorRenderArgs};

use pwt::widget::{GridPicker, Row};

use pwt_macros::builder;

use yew::html::IntoPropValue;
use yew::virtual_dom::{Key, VComp, VNode};

//#[widget(comp=QemuDiskSizeFormatComp, @input)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct QemuDiskSizeFormatSelector {
    /// Onyl allow to select 'raw' format
    #[builder]
    #[prop_or_default]
    pub raw: bool,

    /// Field name used by disk size input ([f64])
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(QemuDiskSizeFormatSelector::DISK_SIZE)]
    pub disk_size_name: AttrValue,

    /// Field name used by disk format input ([String])
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(QemuDiskSizeFormatSelector::DISK_FORMAT)]
    pub disk_format_name: AttrValue,
}

impl QemuDiskSizeFormatSelector {
    pub const DISK_SIZE: AttrValue = AttrValue::Static("_disk_size_");
    pub const DISK_FORMAT: AttrValue = AttrValue::Static("_disk_format_");

    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct QemuDiskSizeFormatComp {
    store: Store<Entry>,
}

#[derive(Clone, PartialEq)]
struct Entry {
    format: String,
    description: String,
}

impl QemuDiskSizeFormatComp {
    fn populate_store(&mut self, ctx: &Context<Self>) {
        let props = ctx.props();

        let mut data = vec![Entry {
            format: String::from("raw"),
            description: tr!("Raw disk image"),
        }];

        if !props.raw {
            data.extend(vec![
                Entry {
                    format: String::from("qcow2"),
                    description: tr!("QEMU image format"),
                },
                Entry {
                    format: String::from("vmdk"),
                    description: tr!("VMware image format"),
                },
            ]);
        }
        self.store.set_data(data);
    }
}

impl Component for QemuDiskSizeFormatComp {
    type Message = ();
    type Properties = QemuDiskSizeFormatSelector;

    fn create(ctx: &Context<Self>) -> Self {
        let store = Store::with_extract_key(|entry: &Entry| Key::from(entry.format.clone()));
        let mut me = Self { store };
        me.populate_store(ctx);
        me
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.raw != old_props.raw {
            self.populate_store(ctx);
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        Row::new()
            .gap(1)
            .with_child(
                Number::<f64>::new()
                    .style("min-width", "0")
                    .name(&props.disk_size_name)
                    .submit(false)
                    .required(true)
                    .min(0.001)
                    .max(128.0 * 1024.0)
                    .default(32.0),
            )
            .with_child(
                Selector::new(
                    self.store.clone(),
                    move |args: &SelectorRenderArgs<Store<Entry>>| {
                        GridPicker::new(
                            DataTable::new(columns(), args.store.clone())
                                .min_width(300)
                                .show_header(false)
                                .header_focusable(false)
                                .class(pwt::css::FlexFit)
                                .into(),
                        )
                        .selection(args.selection.clone())
                        .on_select(args.controller.on_select_callback())
                        .into()
                    },
                )
                .style("min-width", "6em")
                .name(&props.disk_format_name)
                .submit(false)
                .required(true)
                .default("raw")
                .render_value(|v: &AttrValue| v.to_string().into()),
            )
            .into()
    }
}

fn columns() -> Rc<Vec<DataTableHeader<Entry>>> {
    Rc::new(vec![
        DataTableColumn::new(tr!("Format"))
            .width("8em")
            .get_property(|entry: &Entry| &entry.format)
            .into(),
        DataTableColumn::new(tr!("Description"))
            .width("15em")
            .render(|entry: &Entry| entry.description.clone().into())
            .into(),
    ])
}

impl Into<VNode> for QemuDiskSizeFormatSelector {
    fn into(self) -> VNode {
        let comp = VComp::new::<QemuDiskSizeFormatComp>(Rc::new(self), None);
        VNode::from(comp)
    }
}

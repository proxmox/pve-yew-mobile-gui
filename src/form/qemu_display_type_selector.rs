use std::rc::Rc;

use anyhow::format_err;
use indexmap::IndexMap;

use pwt::prelude::*;
use pwt::state::Store;
use pwt::widget::form::{Combobox, ValidateFn};

use pwt::props::{FieldBuilder, WidgetBuilder};
use pwt_macros::{builder, widget};

#[widget(comp=PveQemuDisplayTypeSelector, @input)]
#[derive(Clone, Properties, PartialEq)]
#[builder]
pub struct QemuDisplayTypeSelector {
    /// List of serial device devices ("serial0, serial1, serial2, serial3")
    #[builder]
    #[prop_or_default]
    pub serial_device_list: Option<Rc<Vec<AttrValue>>>,
}

impl QemuDisplayTypeSelector {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct PveQemuDisplayTypeSelector {
    drivers: Rc<IndexMap<AttrValue, String>>,
}

fn kvm_vga_drivers() -> Rc<IndexMap<AttrValue, String>> {
    let mut map: IndexMap<AttrValue, String> = IndexMap::new();

    map.extend([
        (AttrValue::Static("std"), tr!("Standard VGA")),
        (AttrValue::Static("vmware"), tr!("VMware compatible")),
        (AttrValue::Static("qxl"), "SPICE".into()),
        (AttrValue::Static("qxl2"), "SPICE dual monitor".into()),
        (AttrValue::Static("qxl3"), "SPICE three monitors'".into()),
        (AttrValue::Static("qxl4"), "SPICE four monitors".into()),
        (AttrValue::Static("serial0"), tr!("Serial terminal") + " 0"),
        (AttrValue::Static("serial1"), tr!("Serial terminal") + " 1"),
        (AttrValue::Static("serial2"), tr!("Serial terminal") + " 2"),
        (AttrValue::Static("serial3"), tr!("Serial terminal") + " 3"),
        (AttrValue::Static("virtio"), "VirtIO-GPU".into()),
        (AttrValue::Static("virtio-gl"), "VirGL GPU".into()),
        (AttrValue::Static("none"), tr!("none")),
    ]);

    Rc::new(map)
}

pub fn format_qemu_display_type(value: &str) -> String {
    let map = kvm_vga_drivers();
    match map.get(value).cloned() {
        Some(text) => text.clone(),
        None => value.to_string(),
    }
}

impl PveQemuDisplayTypeSelector {
    fn create_validator(
        serial_device_list: Option<Rc<Vec<AttrValue>>>,
    ) -> ValidateFn<(String, Store<AttrValue>)> {
        ValidateFn::new(move |(value, store): &(String, Store<AttrValue>)| {
            if value.starts_with("serial") {
                let empty_list = vec![];
                let serial_device_list = serial_device_list.as_deref().unwrap_or(&empty_list);
                if !serial_device_list.contains(&AttrValue::from(value.clone())) {
                    return Err(format_err!(
                        "Serial interface '{value}' is not correctly configured."
                    ));
                }
            }

            store
                .read()
                .iter()
                .find(|item| item.as_str() == value)
                .ok_or_else(|| format_err!("no such item"))
                .map(|_| ())
        })
    }
}

impl Component for PveQemuDisplayTypeSelector {
    type Message = ();
    type Properties = QemuDisplayTypeSelector;

    fn create(_ctx: &Context<Self>) -> Self {
        let drivers = kvm_vga_drivers();

        Self { drivers }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        Combobox::from_key_value_pairs(self.drivers.as_ref().clone())
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .validate(Self::create_validator(props.serial_device_list.clone()))
            .placeholder(tr!("Default"))
            .show_filter(false)
            .into()
    }
}

use std::collections::HashMap;
use std::rc::Rc;

use yew::html::IntoPropValue;

use pwt::prelude::*;
use pwt::widget::form::Combobox;

use pwt::props::{FieldBuilder, WidgetBuilder};
use pwt_macros::{builder, widget};

#[widget(comp=PveQemuDisplayTypeSelector, @input)]
#[derive(Clone, Properties, PartialEq)]
#[builder]
pub struct QemuDisplayTypeSelector {}

impl QemuDisplayTypeSelector {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct PveQemuDisplayTypeSelector {
    drivers: Rc<HashMap<&'static str, String>>,
    keys: Rc<Vec<AttrValue>>,
}

fn kvm_vga_drivers() -> Rc<HashMap<&'static str, String>> {
    let mut map = HashMap::new();

    map.extend([
        ("std", tr!("Standard VGA")),
        ("vmware", tr!("VMware compatible")),
        ("qxl", "SPICE".into()),
        ("qxl2", "SPICE dual monitor".into()),
        ("qxl3", "SPICE three monitors'".into()),
        ("qxl4", "SPICE four monitors".into()),
        ("serial0", tr!("Serial terminal") + " 0"),
        ("serial1", tr!("Serial terminal") + " 1"),
        ("serial2", tr!("Serial terminal") + " 2"),
        ("serial3", tr!("Serial terminal") + " 3"),
        ("virtio", "VirtIO-GPU".into()),
        ("virtio-gl", "VirGL GPU".into()),
        ("none", tr!("none")),
    ]);

    Rc::new(map)
}

pub fn format_qemu_display_type(value: &str) -> String {
    let map = kvm_vga_drivers();
    render_map_value(&map, value)
}

fn render_map_value(map: &HashMap<&'static str, String>, value: &str) -> String {
    match map.get(value).cloned() {
        Some(text) => text.clone(),
        None => value.to_string(),
    }
}

impl Component for PveQemuDisplayTypeSelector {
    type Message = ();
    type Properties = QemuDisplayTypeSelector;

    fn create(_ctx: &Context<Self>) -> Self {
        let drivers = kvm_vga_drivers();
        let keys: Vec<AttrValue> = drivers.keys().map(|s| AttrValue::from(*s)).collect();

        Self {
            drivers,
            keys: Rc::new(keys),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let map = self.drivers.clone();
        Combobox::new()
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .placeholder(tr!("Default"))
            .show_filter(false)
            .items(Rc::clone(&self.keys))
            .render_value(move |v: &AttrValue| render_map_value(&map, &*v).into())
            .into()
    }
}

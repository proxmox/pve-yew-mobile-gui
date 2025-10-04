use std::rc::Rc;

use indexmap::IndexMap;

use yew::html::IntoPropValue;

use pwt::prelude::*;
use pwt::widget::form::Combobox;

use pwt::props::{FieldBuilder, WidgetBuilder};
use pwt_macros::{builder, widget};

#[widget(comp=PveQemuCacheTypeSelector, @input)]
#[derive(Clone, Properties, PartialEq)]
#[builder]
pub struct QemuCacheTypeSelector {
    /// The default value.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub default: Option<AttrValue>,
}

impl QemuCacheTypeSelector {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct PveQemuCacheTypeSelector {
    items: Rc<IndexMap<AttrValue, String>>,
}

fn qemu_cache_types() -> Rc<IndexMap<AttrValue, String>> {
    let mut items = IndexMap::new();
    items.extend([
        (AttrValue::Static("directsync"), String::from("Direct sync")),
        (
            AttrValue::Static("writethrough"),
            String::from("Write through"),
        ),
        (AttrValue::Static("writeback"), String::from("Write back")),
        (
            AttrValue::Static("unsafe"),
            format!("Write back ({}))", tr!("unsafe")),
        ),
        (AttrValue::Static("none"), tr!("No cache")),
    ]);
    Rc::new(items)
}

/*
pub fn format_qemu_cache_type(value: &str) -> String {
    let map = qemu_cache_types();
    match map.get(value).cloned() {
        Some(text) => text.clone(),
        None => value.to_string(),
    }
}
*/

impl Component for PveQemuCacheTypeSelector {
    type Message = ();
    type Properties = QemuCacheTypeSelector;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            items: qemu_cache_types(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Combobox::from_key_value_pairs(self.items.as_ref().clone())
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .placeholder(tr!("Default") + " (" + &tr!("No cache") + ")")
            .show_filter(false)
            .default(&props.default)
            .into()
    }
}

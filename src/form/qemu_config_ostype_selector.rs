use std::collections::HashMap;
use std::rc::Rc;

use yew::html::IntoPropValue;

use pwt::prelude::*;
use pwt::widget::form::Combobox;

use pwt::props::{FieldBuilder, WidgetBuilder};
use pwt_macros::{builder, widget};

#[widget(comp=PveQemuConfigOstypeSelector, @input)]
#[derive(Clone, Properties, PartialEq)]
#[builder]
pub struct QemuConfigOstypeSelector {
    /// The default value.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub default: Option<AttrValue>,
}

impl QemuConfigOstypeSelector {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn render_value(value: &str) -> String {
        match ITEM_MAP.with(|map| map.get(value).cloned()) {
            Some(text) => text.clone(),
            None => value.to_string(),
        }
    }
}

pub struct PveQemuConfigOstypeSelector {}

impl Component for PveQemuConfigOstypeSelector {
    type Message = ();
    type Properties = QemuConfigOstypeSelector;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        Combobox::new()
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .show_filter(false)
            .default(&props.default)
            .items(ITEM_KEYS.with(Rc::clone))
            .render_value(|v: &AttrValue| QemuConfigOstypeSelector::render_value(&*v).to_html())
            .into()
    }
}

macro_rules! item_list {
    () => {
        [
            ("l26", "Linux 6.x - 2.6 Kernel"),
            ("l24", "Linux 2.4 Kernel"),
            ("win11", "Windows 11/2022/2025"),
            ("win10", "Windows 10/2016/2019"),
            ("win8", "Windows 8.x/2012/2012r2"),
            ("win7", "Windows 7/2008r2"),
            ("w2k8", "Vista/2008"),
            ("wxp", "Windows XP/2003"),
            ("w2k", "Windows 2000"),
            ("solaris", "Solaris Kernel"),
            ("other", "Other"),
        ]
    };
}
thread_local! {
    static ITEM_MAP: HashMap<String, String> = {
        let item_list = item_list!();
        let mut map = HashMap::new();
        for (key, value) in item_list.iter() {
            map.insert(key.to_string(), value.to_string());
        }
        map
    };

    static ITEM_KEYS: Rc<Vec<AttrValue>> = {
        let item_list = item_list!();
        Rc::new(item_list.iter().map(|t| AttrValue::Static(t.0)).collect())
    };
}

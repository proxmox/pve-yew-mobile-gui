use std::collections::HashSet;

use anyhow::Error;
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::{
    Checkbox, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
};
use pwt::widget::Column;

use pwt_macros::{builder, widget};

pub type PveHotplugFeatureSelector = ManagedFieldMaster<PveHotplugFeatureMaster>;

#[widget(comp=ManagedFieldMaster<PveHotplugFeatureMaster>, @input)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct HotplugFeatureSelector {}

impl HotplugFeatureSelector {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub enum Msg {
    SetValue(String, bool),
}

#[doc(hidden)]
pub struct PveHotplugFeatureMaster {
    selection: HashSet<String>,
}

pub fn normalize_hotplug_value(value: &Value) -> Value {
    match value {
        Value::Null => "disk,network,usb".into(),
        Value::String(s) => {
            if s == "0" {
                "".into()
            } else {
                if s == "1" {
                    "disk,network,usb".into()
                } else {
                    s.clone().into()
                }
            }
        }
        _ => value.clone(),
    }
}

impl PveHotplugFeatureMaster {
    pub fn update_selection(&mut self, value: Value) {
        let value = normalize_hotplug_value(&value);
        let value = match value {
            Value::String(s) => s,
            Value::Array(_) => {
                return; // internal state, no update necessary
            }
            _ => {
                log::error!("unable to parse hotplug property string: got wrong type");
                String::new()
            }
        };

        let mut selection = HashSet::new();
        for part in value.split(',') {
            selection.insert(part.to_string());
        }

        self.selection = selection;
    }
}

impl ManagedField for PveHotplugFeatureMaster {
    type Message = Msg;
    type Properties = HotplugFeatureSelector;
    type ValidateClosure = ();

    fn validation_args(_props: &Self::Properties) -> Self::ValidateClosure {
        ()
    }

    fn validator(_props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        let value = match value {
            Value::Array(list) => {
                if list.is_empty() {
                    return Ok("0".into());
                }
                let mut list: Vec<String> = list
                    .iter()
                    .map(|item| item.as_str().map(String::from))
                    .flatten()
                    .filter(|s| !s.is_empty())
                    .collect();

                list.sort();

                list.join(",");

                Value::from(list.join(","))
            }
            _ => value.clone(),
        };

        Ok(value)
    }
    fn setup(_props: &HotplugFeatureSelector) -> ManagedFieldState {
        let value = Value::Null;
        let default = Value::Null;

        let valid = Ok(());

        ManagedFieldState {
            value,
            valid,
            default,
            radio_group: false,
            unique: false,
        }
    }

    fn create(ctx: &ManagedFieldContext<Self>) -> Self {
        let mut me = Self {
            selection: HashSet::new(),
        };
        let state = ctx.state();
        me.update_selection(state.value.clone());
        me
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let state = ctx.state();
        self.update_selection(state.value.clone());
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetValue(name, checked) => {
                if checked {
                    self.selection.insert(name);
                } else {
                    self.selection.remove(&name);
                }
            }
        }
        ctx.link()
            .update_value(serde_json::to_value(&self.selection).unwrap());
        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let cb = |value: &str, label: String| -> Html {
            let checked = self.selection.contains(value);
            let value = value.to_string();
            Checkbox::new()
                .box_label(label)
                .checked(checked)
                .on_input(
                    ctx.link()
                        .callback(move |checked| Msg::SetValue(value.clone(), checked)),
                )
                .into()
        };

        Column::new()
            .class(pwt::css::FlexFit)
            .with_child(cb("disk", tr!("Disk")))
            .with_child(cb("network", tr!("Network")))
            .with_child(cb("usb", String::from("USB")))
            .with_child(cb("memory", tr!("Memory")))
            .with_child(cb("cpu", tr!("CPU")))
            .into()
    }
}

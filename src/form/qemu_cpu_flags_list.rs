use anyhow::Error;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use pwt::prelude::*;
use pwt::widget::form::{
    ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState, RadioButton,
};
use pwt::widget::{Column, List, ListTile, Row};

use pwt_macros::{builder, widget};

pub type QemuCpuFlagsComp = ManagedFieldMaster<QemuCpuFlagsField>;

#[widget(comp=QemuCpuFlagsComp, @input)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct QemuCpuFlags {}

impl QemuCpuFlags {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub enum Msg {
    Set(String, Option<bool>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FlagEntry {
    enabled: Option<bool>,
    name: String,
    descr: String,
}

#[doc(hidden)]
pub struct QemuCpuFlagsField {
    flag_list: IndexMap<&'static str, FlagEntry>,
}

fn parse_flags(flags: &str) -> HashMap<String, bool> {
    flags
        .split(";")
        .filter_map(|flag| {
            if flag.is_empty() {
                None
            } else if flag.starts_with("+") {
                Some((flag[1..].to_string(), true))
            } else if flag.starts_with("-") {
                Some((flag[1..].to_string(), false))
            } else {
                log::error!("unable to parse cpu flag '{flag}' - missing prefix");
                None
            }
        })
        .collect()
}

impl QemuCpuFlagsField {
    pub fn update_flag_list(&mut self, value: Value) {
        let current_flags = parse_flags(value.as_str().unwrap_or(""));
        for (name, item) in self.flag_list.iter_mut() {
            if let Some(value) = current_flags.get(*name) {
                item.enabled = Some(*value);
            } else {
                item.enabled = None;
            }
        }
    }
}

impl ManagedField for QemuCpuFlagsField {
    type Message = Msg;
    type Properties = QemuCpuFlags;
    type ValidateClosure = ();

    fn validation_args(_props: &Self::Properties) -> Self::ValidateClosure {
        ()
    }

    fn validator(_props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        Ok(value.clone())
    }
    fn setup(_props: &QemuCpuFlags) -> ManagedFieldState {
        ManagedFieldState::new(Value::Null, Value::Null)
    }

    fn create(_ctx: &ManagedFieldContext<Self>) -> Self {
        // TODO: let qemu-server host this and autogenerate or get from API call??
        let all_flags = [
            ("md-clear", tr!("Required to let the guest OS know if MDS is mitigated correctly")),
            ("pcid", tr!("Meltdown fix cost reduction on Westmere, Sandy-, and IvyBridge Intel CPUs")),
            ("spec-ctrl", tr!("Allows improved Spectre mitigation with Intel CPUs")),
            ("ssbd", tr!("Protection for \"Speculative Store Bypass\" for Intel models")),
            ("ibpb", tr!("Allows improved Spectre mitigation with AMD CPUs")),
            ("virt-ssbd", tr!("Basis for \"Speculative Store Bypass\" protection for AMD models")), 
            ("amd-ssbd", tr!("Improves Spectre mitigation performance with AMD CPUs, best used with \"virt-ssbd\"")),
            ("amd-no-ssb", tr!("Notifies guest OS that host is not vulnerable for Spectre on AMD CPUs")),
            ("pdpe1gb", tr!("Allow guest OS to use 1GB size pages, if host HW supports it")),
            ("hv-tlbflush", tr!("Improve performance in overcommitted Windows guests. May lead to guest bluescreens on old CPUs.")),
            ("hv-evmcs", tr!("Improve performance for nested virtualization. Only supported on Intel CPUs.")),
            ("aes", tr!("Activate AES instruction set for HW acceleration.")),
        ];

        let mut flag_list = IndexMap::new();
        flag_list.extend(all_flags.into_iter().map(|(flag, descr)| {
            (
                flag,
                FlagEntry {
                    name: flag.to_string(),
                    descr,
                    enabled: None,
                },
            )
        }));

        Self { flag_list }
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let state = ctx.state();
        self.update_flag_list(state.value.clone());
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        let state = ctx.state();
        match msg {
            Msg::Set(flag, enabled) => {
                self.flag_list[flag.as_str()].enabled = enabled;
            }
        }

        let mut new_flags: Vec<String> = self
            .flag_list
            .iter()
            .filter_map(|(name, item)| match item.enabled {
                Some(true) => Some(format!("+{name}")),
                Some(false) => Some(format!("-{name}")),
                None => None,
            })
            .collect();

        // keep unknown flags
        let current_flags = parse_flags(state.value.as_str().unwrap_or(""));
        for (flag, enabled) in current_flags {
            if !self.flag_list.contains_key(flag.as_str()) {
                new_flags.push(match enabled {
                    true => format!("+{flag}"),
                    false => format!("-{flag}"),
                })
            }
        }

        let new_flags = new_flags.join(";");
        ctx.link().update_value(new_flags);
        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let tiles: Vec<ListTile> = self
            .flag_list
            .iter()
            .enumerate()
            .map(|(index, (name, item))| {
                let is_last = (index + 1) == self.flag_list.len();

                let status = match item.enabled {
                    Some(true) => tr!("On"),
                    Some(false) => tr!("Off"),
                    None => tr!("Default"),
                };

                let trailing: Html = Row::new()
                    .class(pwt::css::AlignItems::Center)
                    .with_child(
                        RadioButton::new("off")
                            .checked(item.enabled == Some(false))
                            .on_input(ctx.link().callback({
                                let name = name.to_string();
                                move |_| Msg::Set(name.clone(), Some(false))
                            })),
                    )
                    .with_child(
                        RadioButton::new("default")
                            .checked(item.enabled == None)
                            .on_input(ctx.link().callback({
                                let name = name.to_string();
                                move |_| Msg::Set(name.clone(), None)
                            })),
                    )
                    .with_child(
                        RadioButton::new("on")
                            .checked(item.enabled == Some(true))
                            .on_input(ctx.link().callback({
                                let name = name.to_string();
                                move |_| Msg::Set(name.clone(), Some(true))
                            })),
                    )
                    .into();

                let trailing: Html = Column::new()
                    .class(pwt::css::AlignItems::Center)
                    .with_child(trailing)
                    .with_child(status)
                    .into();

                crate::widgets::form_list_tile(item.name.clone(), item.descr.clone(), trailing)
                    .interactive(true)
                    .border_bottom(!is_last)
                    .padding_x(0)
                    .key(item.name.clone())
            })
            .collect();
        List::from_tiles(tiles)
            .class(pwt::css::FlexFit)
            .grid_template_columns("1fr auto")
            .into()
    }
}

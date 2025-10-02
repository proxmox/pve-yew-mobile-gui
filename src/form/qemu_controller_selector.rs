use anyhow::{bail, Error};
use serde_json::{json, Value};

use pwt::prelude::*;
use pwt::widget::form::{
    Combobox, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState, Number,
};
use pwt::widget::Row;

use pwt_macros::{builder, widget};

use pve_api_types::{
    QemuConfigIdeArray, QemuConfigSataArray, QemuConfigScsiArray, QemuConfigVirtioArray,
};

pub type QemuControllerSelectorComp = ManagedFieldMaster<QemuControllerSelectorField>;

#[widget(comp=QemuControllerSelectorComp, @input)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct QemuControllerSelector {}

impl QemuControllerSelector {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct QemuControllerSelectorField {
    controller: String,
    device_id: String,
}

pub enum Msg {
    SetDeviceId(String),
    SetController(String),
}

pub fn parse_qemu_controller_name(s: &str) -> Result<(&'static str, u32), Error> {
    if let Some(rest) = s.strip_prefix("ide") {
        Ok(("ide", rest.parse::<u32>()?))
    } else if let Some(rest) = s.strip_prefix("sata") {
        Ok(("sata", rest.parse::<u32>()?))
    } else if let Some(rest) = s.strip_prefix("scsi") {
        Ok(("scsi", rest.parse::<u32>()?))
    } else if let Some(rest) = s.strip_prefix("virtio") {
        Ok(("virtio", rest.parse::<u32>()?))
    } else {
        bail!("unable to parse device name '{s}'");
    }
}

impl ManagedField for QemuControllerSelectorField {
    type Message = Msg;
    type Properties = QemuControllerSelector;
    type ValidateClosure = ();

    fn validation_args(_props: &Self::Properties) -> Self::ValidateClosure {
        ()
    }

    fn validator(_props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        match value {
            Value::String(s) => {
                let _ = parse_qemu_controller_name(s)?;
                Ok(s.to_string().into())
            }
            _ => {
                bail!("invalid device name");
            }
        }
    }

    fn setup(_props: &Self::Properties) -> ManagedFieldState {
        ManagedFieldState::new(Value::Null, Value::Null)
    }

    fn create(_ctx: &ManagedFieldContext<Self>) -> Self {
        Self {
            controller: String::new(),
            device_id: String::new(),
        }
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let state = ctx.state();
        match &state.value {
            Value::String(s) => match parse_qemu_controller_name(s) {
                Ok((controller, id)) => {
                    self.controller = controller.into();
                    self.device_id = id.to_string();
                }
                Err(err) => {
                    self.controller = String::new();
                    self.device_id = String::new();
                    log::error!("QemuControllerSelector - got invalid value: {err}")
                }
            },
            Value::Object(_) => { /* internal state */ }
            Value::Null => {
                self.controller = String::new();
                self.device_id = String::new();
            }
            _ => {
                self.controller = String::new();
                self.device_id = String::new();
                log::error!("QemuControllerSelector - got invalid value type")
            }
        }
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetController(controller) => {
                self.controller = controller;
            }
            Msg::SetDeviceId(device_id) => {
                self.device_id = device_id;
            }
        }

        let device = format!("{}{}", self.controller, self.device_id);
        let new_value: Value = if let Ok(_) = parse_qemu_controller_name(&device) {
            device.into()
        } else {
            json!({"controller": self.controller, "device_id": self.device_id})
        };
        ctx.link().update_value(new_value);
        true
    }
    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let max: Option<u32> = match self.controller.as_str() {
            "ide" => Some(QemuConfigIdeArray::MAX as u32 - 1),
            "sata" => Some(QemuConfigSataArray::MAX as u32 - 1),
            "scsi" => Some(QemuConfigScsiArray::MAX as u32 - 1),
            "virtio" => Some(QemuConfigVirtioArray::MAX as u32 - 1),
            _ => None,
        };
        Row::new()
            .with_child(
                Combobox::new()
                    .required(true)
                    .force_selection(true)
                    .value(self.controller.clone())
                    .with_item("ide")
                    .with_item("scsi")
                    .with_item("sata")
                    .on_change(ctx.link().callback(Msg::SetController)),
            )
            .with_child(
                Number::<u32>::new()
                    .required(true)
                    .min(0)
                    .max(max)
                    .value(self.device_id.clone())
                    .on_input(ctx.link().callback(|(s, _)| Msg::SetDeviceId(s))),
            )
            .into()
    }
}

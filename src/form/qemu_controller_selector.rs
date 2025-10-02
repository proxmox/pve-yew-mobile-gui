use std::collections::HashSet;
use std::rc::Rc;

use anyhow::{bail, Error};
use serde_json::{json, Value};

use pwt::prelude::*;
use pwt::widget::form::{
    Combobox, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState, Number,
    ValidateFn,
};
use pwt::widget::Row;

use pwt_macros::{builder, widget};

use pve_api_types::{
    QemuConfigIdeArray, QemuConfigSataArray, QemuConfigScsiArray, QemuConfigVirtioArray,
};
use yew::html::IntoPropValue;

pub type QemuControllerSelectorComp = ManagedFieldMaster<QemuControllerSelectorField>;

#[widget(comp=QemuControllerSelectorComp, @input)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct QemuControllerSelector {
    #[builder]
    #[prop_or_default]
    pub allow_virtio: bool,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub exclude_devices: Option<HashSet<String>>,
}

impl QemuControllerSelector {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub struct QemuControllerSelectorField {
    controller: String,
    device_id: String,
    validate: ValidateFn<u32>,
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

fn create_validator(props: &QemuControllerSelector, controller: &str) -> ValidateFn<u32> {
    let controller = controller.to_string();
    let props = props.clone();
    ValidateFn::new(move |id: &u32| {
        if controller.is_empty() {
            bail!("no controller selected");
        }
        let device_name = format!("{}{}", controller, id);
        let _ = parse_qemu_controller_name(&device_name)?;

        if let Some(exclude_devices) = &props.exclude_devices {
            if exclude_devices.contains(&device_name) {
                bail!(tr!("device '{0}' name is already in use.", device_name));
            }
        }

        Ok(())
    })
}

impl ManagedField for QemuControllerSelectorField {
    type Message = Msg;
    type Properties = QemuControllerSelector;
    type ValidateClosure = QemuControllerSelector;

    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure {
        props.clone()
    }

    fn setup(_props: &Self::Properties) -> ManagedFieldState {
        ManagedFieldState::new(Value::Null, Value::Null)
    }

    fn create(ctx: &ManagedFieldContext<Self>) -> Self {
        Self {
            controller: String::new(),
            device_id: String::new(),
            validate: create_validator(ctx.props(), ""),
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
        let new_value: Value = match parse_qemu_controller_name(&device) {
            Ok(_) => device.into(),
            Err(_) => json!({"controller": self.controller, "device_id": self.device_id}),
        };

        self.validate = create_validator(ctx.props(), &self.controller);
        ctx.link().update_value(new_value);

        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let max: Option<u32> = match self.controller.as_str() {
            "ide" => Some(QemuConfigIdeArray::MAX as u32 - 1),
            "sata" => Some(QemuConfigSataArray::MAX as u32 - 1),
            "scsi" => Some(QemuConfigScsiArray::MAX as u32 - 1),
            "virtio" => Some(QemuConfigVirtioArray::MAX as u32 - 1),
            _ => None,
        };

        let mut items = vec![
            AttrValue::Static("ide"),
            AttrValue::Static("sata"),
            AttrValue::Static("scsi"),
        ];
        if props.allow_virtio {
            items.push(AttrValue::Static("virtio"));
        }

        Row::new()
            .with_child(
                Combobox::new()
                    .required(true)
                    .force_selection(true)
                    .value(self.controller.clone())
                    .items(Rc::new(items))
                    .on_change(ctx.link().callback(Msg::SetController)),
            )
            .with_child(
                Number::<u32>::new()
                    .required(true)
                    .min(0)
                    .max(max)
                    .value(self.device_id.clone())
                    .validate(self.validate.clone())
                    .on_input(ctx.link().callback(|(s, _)| Msg::SetDeviceId(s))),
            )
            .into()
    }
}

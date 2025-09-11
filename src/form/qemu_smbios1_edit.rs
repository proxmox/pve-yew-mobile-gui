use anyhow::{bail, Error};
use regex::Regex;
use serde_json::{json, Value};

use proxmox_schema::ApiType;
use pve_api_types::PveQmSmbios1;

use pwt::prelude::*;
use pwt::widget::form::{
    Field, ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState, TextArea,
};
use pwt::widget::Column;

use pwt_macros::{builder, widget};

pub type PveQemuSmbios1Edit = ManagedFieldMaster<PveQemuSmbios1Master>;

#[widget(comp=ManagedFieldMaster<PveQemuSmbios1Master>, @input)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct QemuSmbios1Edit {}

impl QemuSmbios1Edit {
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub enum Msg {
    SetUUID(String),
    SetManufacturer(String),
    SetProduct(String),
    SetVersion(String),
    SetSerial(String),
    SetSKU(String),
    SetFamily(String),
}

#[doc(hidden)]
pub struct PveQemuSmbios1Master {
    data: PveQmSmbios1,
}

fn smbios1_default() -> PveQmSmbios1 {
    // todo: implement PveQmSmbios1::default() ?
    serde_json::from_value(json!({})).unwrap()
}

fn smbios1_decode(mut data: PveQmSmbios1) -> PveQmSmbios1 {
    let decode_option = |target: &mut Option<String>| {
        if let Some(data) = target.as_deref() {
            if let Ok(bin_data) = proxmox_base64::decode(data) {
                *target = Some(String::from_utf8_lossy(&bin_data).into());
            }
        }
    };

    if let Some(true) = data.base64 {
        data.base64 = Some(false);
        decode_option(&mut data.manufacturer);
        decode_option(&mut data.product);
        decode_option(&mut data.version);
        decode_option(&mut data.serial);
        decode_option(&mut data.sku);
        decode_option(&mut data.family);
    }

    data
}

fn smbios1_encode(mut data: PveQmSmbios1) -> PveQmSmbios1 {
    let mut use_base64 = false;
    let mut encode_option = |target: &mut Option<String>| {
        if let Some(data) = target.as_deref() {
            use_base64 = true;
            *target = Some(proxmox_base64::encode(data));
        }
    };

    match data.base64 {
        None | Some(false) => {
            encode_option(&mut data.manufacturer);
            encode_option(&mut data.product);
            encode_option(&mut data.version);
            encode_option(&mut data.serial);
            encode_option(&mut data.sku);
            encode_option(&mut data.family);

            data.base64 = use_base64.then(|| true);
        }
        _ => { /* do nothing */ }
    }

    data
}

thread_local! {
    static UUID_MATCH: Regex = Regex::new(r#"^[a-fA-F0-9]{8}(?:-[a-fA-F0-9]{4}){3}-[a-fA-F0-9]{12}$"#).unwrap();
}
impl PveQemuSmbios1Master {
    pub fn update_data(&mut self, value: Value) {
        match value {
            Value::Null => self.data = smbios1_default(),
            Value::String(s) => match PveQmSmbios1::API_SCHEMA.parse_property_string(&s) {
                Ok(value) => self.data = smbios1_decode(serde_json::from_value(value).unwrap()),
                Err(err) => {
                    log::error!("unable to parse smbios1 property string: {err}");
                    self.data = smbios1_default();
                }
            },
            Value::Object(_) => {
                return; // internal state, no update necessary
            }
            _ => {
                log::error!("unable to parse hotplug property string: got wrong type");
                self.data = smbios1_default();
            }
        };
    }
}

impl ManagedField for PveQemuSmbios1Master {
    type Message = Msg;
    type Properties = QemuSmbios1Edit;
    type ValidateClosure = ();

    fn validation_args(_props: &Self::Properties) -> Self::ValidateClosure {
        ()
    }

    fn validator(_props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        let value = match value {
            Value::Object(_) => {
                let data = smbios1_encode(serde_json::from_value(value.clone())?);
                let text = proxmox_schema::property_string::print::<PveQmSmbios1>(&data)?;
                text.into()
            }
            _ => value.clone(),
        };

        Ok(value)
    }
    fn setup(_props: &QemuSmbios1Edit) -> ManagedFieldState {
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
            data: smbios1_default(),
        };
        let state = ctx.state();
        me.update_data(state.value.clone());
        me
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let state = ctx.state();
        self.update_data(state.value.clone());
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        let into_option = |text: String| {
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        };

        match msg {
            Msg::SetUUID(uuid) => self.data.uuid = into_option(uuid),
            Msg::SetManufacturer(manufacturer) => {
                self.data.manufacturer = into_option(manufacturer)
            }
            Msg::SetProduct(product) => self.data.product = into_option(product),
            Msg::SetVersion(version) => self.data.version = into_option(version),
            Msg::SetSerial(serial) => self.data.serial = into_option(serial),
            Msg::SetSKU(sku) => self.data.sku = into_option(sku),
            Msg::SetFamily(family) => self.data.family = into_option(family),
        }
        ctx.link()
            .update_value(serde_json::to_value(&self.data).unwrap());
        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let field_height = "2em";

        Column::new()
            .gap(2)
            // This is scrollable, so we Diasble the SideDialog gesture detecture..
            .onpointerdown(|event: PointerEvent| {
                event.stop_propagation();
            })
            .ontouchstart(|event: TouchEvent| {
                event.stop_propagation();
            })
            .class(pwt::css::FlexFit)
            .class(pwt::css::AlignItems::Stretch)
            .with_child(crate::widgets::label_field(
                tr!("UUID"),
                Field::new()
                    .value(self.data.uuid.clone())
                    .on_input(ctx.link().callback(Msg::SetUUID))
                    .validate(|v: &String| {
                        if UUID_MATCH.with(|r| r.is_match(v)) {
                            return Ok(());
                        }
                        bail!(
                            tr!("Format")
                                + ": xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx (where x is 0-9 or a-f or A-F)"
                        )
                    }),
            ))
            .with_child(crate::widgets::label_field(
                tr!("Manufacturer"),
                TextArea::new()
                    .style("height", field_height)
                    .value(self.data.manufacturer.clone())
                    .on_input(ctx.link().callback(Msg::SetManufacturer)),
            ))
            .with_child(crate::widgets::label_field(
                tr!("Product"),
                TextArea::new()
                    .style("height", field_height)
                    .value(self.data.product.clone())
                    .on_input(ctx.link().callback(Msg::SetProduct)),
            ))
            .with_child(crate::widgets::label_field(
                tr!("Version"),
                TextArea::new()
                    .style("height", field_height)
                    .value(self.data.version.clone())
                    .on_input(ctx.link().callback(Msg::SetVersion)),
            ))
            .with_child(crate::widgets::label_field(
                tr!("Serial"),
                TextArea::new()
                    .style("height", field_height)
                    .value(self.data.serial.clone())
                    .on_input(ctx.link().callback(Msg::SetSerial)),
            ))
            .with_child(crate::widgets::label_field(
                "SKU",
                TextArea::new()
                    .style("height", field_height)
                    .value(self.data.sku.clone())
                    .on_input(ctx.link().callback(Msg::SetSKU)),
            ))
            .with_child(crate::widgets::label_field(
                tr!("Family"),
                TextArea::new()
                    .style("height", field_height)
                    .value(self.data.family.clone())
                    .on_input(ctx.link().callback(Msg::SetFamily)),
            ))
            .into()
    }
}

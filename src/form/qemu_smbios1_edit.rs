use anyhow::bail;
use proxmox_yew_comp::form::property_string_from_parts;
use pwt::props::SubmitCallback;
use regex::Regex;
use serde_json::Value;

use proxmox_schema::ApiType;

use pve_api_types::{PveQmSmbios1, QemuConfig};

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Field, FormContext, TextArea};
use pwt::widget::Column;

use proxmox_yew_comp::{http_put, ApiLoadCallback};

use crate::form::{flatten_property_string, typed_load};
use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};

thread_local! {
    static UUID_MATCH: Regex = Regex::new(r#"^[a-fA-F0-9]{8}(?:-[a-fA-F0-9]{4}){3}-[a-fA-F0-9]{12}$"#).unwrap();
}

// All base64 encodable properties (without "uuid")
const PROPERTIES: &[&str] = &[
    "manufacturer",
    "product",
    "version",
    "serial",
    "sku",
    "family",
];

fn input_panel(name: String) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_, _| {
        let field_height = "2em";
        let property_name = |prop| format!("_{name}_{prop}");

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
                            .name(property_name("uuid"))
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
                            .name(property_name("manufacturer"))
                            .style("height", field_height)
                    ))
                    .with_child(crate::widgets::label_field(
                        tr!("Product"),
                        TextArea::new()
                            .name(property_name("product"))
                            .style("height", field_height)
                    ))
                    .with_child(crate::widgets::label_field(
                        tr!("Version"),
                        TextArea::new()
                            .name(property_name("version"))
                            .style("height", field_height)
                    ))
                    .with_child(crate::widgets::label_field(
                        tr!("Serial"),
                        TextArea::new()
                            .name(property_name("serial"))
                    .style("height", field_height)
            ))
            .with_child(crate::widgets::label_field(
                "SKU",
                TextArea::new()
                    .name(property_name("sku"))
                    .style("height", field_height)
            ))
            .with_child(crate::widgets::label_field(
                tr!("Family"),
                TextArea::new()
                    .name(property_name("family"))
                    .style("height", field_height)
            ))
            .into()
    })
}

pub fn qemu_smbios_property(name: impl Into<String>, url: impl Into<String>) -> EditableProperty {
    let url = url.into();
    let name = name.into();
    EditableProperty::new(name.clone(), tr!("SMBIOS settings (type1)"))
        .required(true)
        .render_input_panel(input_panel(name.clone()))
        .loader({
            let cloned_url = url.clone();
            let name = name.clone();
            ApiLoadCallback::new(move || {
                let cloned_url = cloned_url.clone();
                let name = name.clone();
                async move {
                    let property_name = |prop| format!("_{name}_{prop}");
                    let mut resp = typed_load::<QemuConfig>(cloned_url).apply().await?;
                    flatten_property_string(&mut resp.data, &name, &PveQmSmbios1::API_SCHEMA);
                    if let Some(Value::Bool(true)) = resp.data.get(property_name("base64")) {
                        for prop in PROPERTIES.iter().map(|prop| property_name(prop)) {
                            if let Some(Value::String(base64)) = resp.data.get(&prop) {
                                if let Ok(bin_data) = proxmox_base64::decode(base64) {
                                    resp.data[prop] = String::from_utf8_lossy(&bin_data).into();
                                }
                            }
                        }
                    }
                    Ok(resp)
                }
            })
            .url(url.clone())
        })
        .on_submit({
            let url = url.clone();
            SubmitCallback::new(move |ctx: FormContext| {
                let url = url.clone();
                let name = name.clone();
                async move {
                    let mut value = ctx.get_submit_data();
                    let mut base64 = false;
                    let property_name = |prop| format!("_{name}_{prop}");

                    for name in PROPERTIES.iter().map(|n| property_name(*n)) {
                        if let Some(Value::String(utf8)) = value.get(&name) {
                            base64 = true;
                            value[name] = proxmox_base64::encode(utf8).into();
                        }
                    }
                    if base64 {
                        value[property_name("base64")] = true.into();
                    }
                    property_string_from_parts::<PveQmSmbios1>(&mut value, &name, true);
                    let value = delete_empty_values(&value, &[&name], false);
                    http_put(url.clone(), Some(value)).await
                }
            })
        })
}

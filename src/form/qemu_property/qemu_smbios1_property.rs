use anyhow::bail;
use regex::Regex;
use serde_json::Value;

use pve_api_types::PveQmSmbios1;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Field, TextArea};
use pwt::widget::Column;

use crate::form::{flatten_property_string, property_string_from_parts};
use crate::widgets::{EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn};

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

fn input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_| {
        let field_height = "2em";
        Column::new()
                    .gap(2)
                    .class(pwt::css::FlexFit)
                    .class(pwt::css::AlignItems::Stretch)
                    .with_child(crate::widgets::label_field(
                        tr!("UUID"),
                        Field::new()
                            .name("_uuid")
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
                            .name("_manufacturer")
                            .style("height", field_height)
                    ))
                    .with_child(crate::widgets::label_field(
                        tr!("Product"),
                        TextArea::new()
                            .name("_product")
                            .style("height", field_height)
                    ))
                    .with_child(crate::widgets::label_field(
                        tr!("Version"),
                        TextArea::new()
                            .name("_version")
                            .style("height", field_height)
                    ))
                    .with_child(crate::widgets::label_field(
                        tr!("Serial"),
                        TextArea::new()
                            .name("_serial")
                    .style("height", field_height)
            ))
            .with_child(crate::widgets::label_field(
                "SKU",
                TextArea::new()
                    .name("_sku")
                    .style("height", field_height)
            ))
            .with_child(crate::widgets::label_field(
                tr!("Family"),
                TextArea::new()
                    .name("_family")
                    .style("height", field_height)
            ))
            .into()
    })
}

pub fn qemu_smbios_property() -> EditableProperty {
    EditableProperty::new("smbios1", tr!("SMBIOS settings (type1)"))
        .required(true)
        .render_input_panel(input_panel())
        .load_hook(move |mut record: Value| {
            flatten_property_string::<PveQmSmbios1>(&mut record, "smbios1")?;

            // decode base64 encoded properties
            if let Some(Value::Bool(true)) = record.get("_base64") {
                for prop in PROPERTIES.iter().map(|prop| format!("_{prop}")) {
                    if let Some(Value::String(base64)) = record.get(&prop) {
                        if let Ok(bin_data) = proxmox_base64::decode(base64) {
                            record[prop] = String::from_utf8_lossy(&bin_data).into();
                        }
                    }
                }
            }
            Ok(record)
        })
        .submit_hook(move |state: PropertyEditorState| {
            let mut value = state.get_submit_data();
            let mut base64 = false;

            // always base64 encoded properties
            for name in PROPERTIES.iter().map(|n| format!("_{n}")) {
                if let Some(Value::String(utf8)) = value.get(&name) {
                    base64 = true;
                    value[name] = proxmox_base64::encode(utf8).into();
                }
            }
            if base64 {
                value["_base64"] = true.into();
            }
            property_string_from_parts::<PveQmSmbios1>(&mut value, "smbios1", true)?;
            let value = delete_empty_values(&value, &["smbios1"], false);
            Ok(value)
        })
}

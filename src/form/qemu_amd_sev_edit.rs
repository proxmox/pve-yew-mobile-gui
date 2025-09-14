use serde_json::{json, Value};

use proxmox_schema::ApiType;
use proxmox_yew_comp::{
    form::{flatten_property_string, property_string_from_parts},
    http_put, ApiLoadCallback,
};
use pve_api_types::{PveQemuSevFmt, PveQemuSevFmtType, QemuConfig};

use pwt::{
    prelude::*,
    props::SubmitCallback,
    widget::{
        form::{delete_empty_values, Checkbox, Combobox, FormContext},
        Column, Container,
    },
};

use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};

fn input_panel(name: String) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, _| {
        let property_name = |prop| format!("_{name}_{prop}");
        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        let amd_sev_type = form_ctx.read().get_field_text(property_name("type"));
        let snp_enabled = amd_sev_type == "snp";
        let sev_enabled = !amd_sev_type.is_empty();

        Column::new()
            .gap(2)
            .with_child(
                Combobox::new()
                    .name(property_name("type"))
                    .with_item("std")
                    .with_item("es")
                    .with_item("snp")
                    .placeholder(format!("{} ({})", tr!("Default"), tr!("Disabled")))
                    .render_value(|v: &AttrValue| {
                        match v.as_str() {
                            "std" => "AMD SEV",
                            "es" => "AMD SEV-ES (highly experimental)",
                            "snp" => "AMD SEV-SNP (highly experimental)",
                            _ => v,
                        }
                        .into()
                    }),
            )
            .with_child(
                Checkbox::new()
                    .style("display", (!sev_enabled).then(|| "none"))
                    .disabled(!sev_enabled)
                    .submit(false)
                    .name(property_name("debug"))
                    .box_label(tr!("Allow Debugging")),
            )
            .with_child(
                Checkbox::new()
                    .style("display", (!sev_enabled || snp_enabled).then(|| "none"))
                    .disabled(!sev_enabled || snp_enabled)
                    .submit(false)
                    .name(property_name("key-sharing"))
                    .box_label(tr!("Allow Key-Sharing")),
            )
            .with_child(
                Checkbox::new()
                    .style("display", (!sev_enabled).then(|| "none"))
                    .disabled(!sev_enabled)
                    .name(property_name("kernel-hashes"))
                    .submit(false)
                    .box_label(tr!("Enable Kernel Hashes")),
            )
            .with_optional_child(snp_enabled.then(|| {
                hint(tr!(
                    "WARNING: When using SEV-SNP no EFI disk is loaded as pflash."
                ))
            }))
            .with_optional_child(snp_enabled.then(|| {
                hint(tr!(
                    "Note: SEV-SNP requires host kernel version 6.11 or higher."
                ))
            }))
            .into()
    })
}

pub fn qemu_amd_sev_property(name: impl Into<String>, url: impl Into<String>) -> EditableProperty {
    let url = url.into();
    let name = name.into();

    EditableProperty::new("amd-sev", tr!("AMD SEV"))
        .required(true)
        .placeholder(format!("{} ({})", tr!("Default"), tr!("Disabled")))
        .render_input_panel(input_panel(name.clone()))
        .renderer(|_, v, _| {
            if let serde_json::Value::String(v) = v {
                if let Ok(data) = crate::form::parse_property_string::<PveQemuSevFmt>(v) {
                    let text = match data.ty {
                        PveQemuSevFmtType::Std => "AMD SEV",
                        PveQemuSevFmtType::Es => "AMD SEV-ES",
                        PveQemuSevFmtType::Snp => "AMD SEV-SNP",
                    };
                    return format!("{text} ({v})").into();
                }
            }
            v.into()
        })
        .loader(crate::form::load_property_string::<QemuConfig, PveQemuSevFmt>(&url, &name))
        .loader({
            let url_cloned = url.clone();
            let name = name.clone();
            ApiLoadCallback::new(move || {
                let url = url_cloned.clone();
                let name = name.clone();
                async move {
                    let property_name = |prop| format!("_{name}_{prop}");

                    let mut resp = crate::form::typed_load::<QemuConfig>(url).apply().await?;
                    flatten_property_string(&mut resp.data, &name, &PveQemuSevFmt::API_SCHEMA);

                    let no_debug = resp
                        .data
                        .get(property_name("no-debug"))
                        .unwrap_or(&Value::Bool(false))
                        .as_bool()
                        .unwrap_or(false);
                    resp.data[property_name("debug")] = (!no_debug).into();

                    let no_key_sharing = resp
                        .data
                        .get(property_name("no-key-sharing"))
                        .unwrap_or(&Value::Bool(false))
                        .as_bool()
                        .unwrap_or(false);
                    resp.data[property_name("key-sharing")] = (!no_key_sharing).into();

                    Ok(resp)
                }
            })
            .url(url.clone())
        })
        .on_submit({
            let url = url.clone();
            SubmitCallback::new(move |form_ctx: FormContext| {
                let url = url.clone();
                let name = name.clone();
                async move {
                    let property_name = |prop| format!("_{name}_{prop}");
                    let mut form_data = form_ctx.get_submit_data();
                    let ty = match form_data.get(property_name("type")) {
                        Some(Value::String(ty)) => ty.clone(),
                        _ => String::new(),
                    };
                    if ty.is_empty() {
                        let value = json!({"delete": name});
                        return http_put(url, Some(value)).await;
                    }

                    let debug = form_ctx.read().get_field_checked(property_name("debug"));
                    if !debug {
                        form_data[property_name("no-debug")] = true.into();
                    }

                    let key_sharing = form_ctx
                        .read()
                        .get_field_checked(property_name("key-sharing"));
                    if !key_sharing && ty != "snp" {
                        form_data[property_name("no-key-sharing")] = true.into();
                    }

                    let kernel_hashes_name = property_name("kernel-hashes");
                    if form_ctx
                        .read()
                        .get_field_checked(kernel_hashes_name.clone())
                    {
                        form_data[kernel_hashes_name] = true.into();
                    }

                    property_string_from_parts::<PveQemuSevFmt>(&mut form_data, &name, true);
                    let value = delete_empty_values(&form_data, &[&name], false);
                    http_put(url, Some(value)).await
                }
            })
        })
}

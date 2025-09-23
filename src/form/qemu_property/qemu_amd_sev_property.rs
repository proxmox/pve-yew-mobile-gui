use proxmox_schema::property_string::PropertyString;
use serde_json::{json, Value};

use proxmox_schema::ApiType;
use pve_api_types::{PveQemuSevFmt, PveQemuSevFmtType};

use pwt::prelude::*;

use pwt::widget::form::{delete_empty_values, Checkbox, Combobox, FormContext};
use pwt::widget::{Column, Container};

use crate::form::{flatten_property_string, property_string_from_parts, pspn};
use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};

fn input_panel(name: String) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, _| {
        let advanced = form_ctx.get_show_advanced();

        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        let amd_sev_type = form_ctx.read().get_field_text(pspn(&name, "type"));
        let snp_enabled = amd_sev_type == "snp";
        let sev_enabled = !amd_sev_type.is_empty();

        Column::new()
            .gap(2)
            .padding_bottom(1) // avoid scrollbar ?!
            .with_child(
                Combobox::from_key_value_pairs([
                    ("std", "AMD SEV"),
                    ("es", "AMD SEV-ES (highly experimental)"),
                    ("snp", "AMD SEV-SNP (highly experimental)"),
                ])
                .name(pspn(&name, "type"))
                .force_selection(true)
                .placeholder(format!("{} ({})", tr!("Default"), tr!("Disabled"))),
            )
            .with_child(
                Checkbox::new()
                    .class((!advanced || !sev_enabled).then(|| pwt::css::Display::None))
                    .disabled(!sev_enabled)
                    .submit(false)
                    .name(pspn(&name, "debug"))
                    .box_label(tr!("Allow Debugging")),
            )
            .with_child(
                Checkbox::new()
                    .class(
                        (!advanced || !sev_enabled || snp_enabled).then(|| pwt::css::Display::None),
                    )
                    .disabled(!sev_enabled || snp_enabled)
                    .submit(false)
                    .name(pspn(&name, "key-sharing"))
                    .box_label(tr!("Allow Key-Sharing")),
            )
            .with_child(
                Checkbox::new()
                    .class((!advanced || !snp_enabled).then(|| pwt::css::Display::None))
                    .disabled(!snp_enabled)
                    .default(true)
                    .submit(false)
                    .name(pspn(&name, "allow-smt"))
                    .box_label(tr!("Allow SMT")),
            )
            .with_child(
                Checkbox::new()
                    .class((!advanced || !sev_enabled).then(|| pwt::css::Display::None))
                    .disabled(!sev_enabled)
                    .name(pspn(&name, "kernel-hashes"))
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

pub fn qemu_amd_sev_property(name: impl Into<String>) -> EditableProperty {
    let name = name.into();

    EditableProperty::new("amd-sev", tr!("AMD SEV"))
        .advanced_checkbox(true)
        .required(true)
        .placeholder(format!("{} ({})", tr!("Default"), tr!("Disabled")))
        .render_input_panel(input_panel(name.clone()))
        .renderer(|_, v, _| {
            match serde_json::from_value::<Option<PropertyString<PveQemuSevFmt>>>(v.clone()) {
                Ok(Some(data)) => {
                    let text = match data.ty {
                        PveQemuSevFmtType::Std => "AMD SEV",
                        PveQemuSevFmtType::Es => "AMD SEV-ES",
                        PveQemuSevFmtType::Snp => "AMD SEV-SNP",
                    };
                    format!("{text} ({v})").into()
                }
                _ => v.into(),
            }
        })
        .load_hook({
            let name = name.clone();
            move |mut record| {
                flatten_property_string(&mut record, &name, &PveQemuSevFmt::API_SCHEMA)?;

                let no_debug = record[pspn(&name, "no-debug")].as_bool().unwrap_or(false);
                record[pspn(&name, "debug")] = (!no_debug).into();

                let no_key_sharing = record[pspn(&name, "no-key-sharing")]
                    .as_bool()
                    .unwrap_or(false);
                record[pspn(&name, "key-sharing")] = (!no_key_sharing).into();

                Ok(record)
            }
        })
        .submit_hook({
            let name = name.clone();
            move |form_ctx: FormContext| {
                let mut form_data = form_ctx.get_submit_data();
                let ty = match form_data.get(pspn(&name, "type")) {
                    Some(Value::String(ty)) => ty.clone(),
                    _ => String::new(),
                };
                if ty.is_empty() {
                    return Ok(json!({"delete": name}));
                }

                let debug = form_ctx.read().get_field_checked(pspn(&name, "debug"));
                if !debug {
                    form_data[pspn(&name, "no-debug")] = true.into();
                }

                let key_sharing = form_ctx
                    .read()
                    .get_field_checked(pspn(&name, "key-sharing"));
                if !key_sharing && ty != "snp" {
                    form_data[pspn(&name, "no-key-sharing")] = true.into();
                }

                let allow_smt_name = pspn(&name, "allow-smt");
                let allow_smt = form_ctx.read().get_field_checked(allow_smt_name.clone());
                if !allow_smt && ty == "snp" {
                    form_data[allow_smt_name] = false.into();
                }

                let kernel_hashes_name = pspn(&name, "kernel-hashes");
                if form_ctx
                    .read()
                    .get_field_checked(kernel_hashes_name.clone())
                {
                    form_data[kernel_hashes_name] = true.into();
                }

                property_string_from_parts::<PveQemuSevFmt>(&mut form_data, &name, true)?;
                let form_data = delete_empty_values(&form_data, &[&name], false);
                Ok(form_data)
            }
        })
}

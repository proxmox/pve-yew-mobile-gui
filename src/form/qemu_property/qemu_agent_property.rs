use proxmox_schema::property_string::PropertyString;
use proxmox_yew_comp::utils::render_boolean;
use pve_api_types::QemuConfigAgent;

use pwt::prelude::*;
use pwt::widget::form::{Checkbox, Combobox};
use pwt::widget::Column;
use pwt::widget::{form::FormContext, Container};
use serde_json::Value;

use crate::form::{property_string_load_hook, property_string_submit_hook, pspn};
use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};

fn renderer(_name: &str, value: &Value, _record: &Value) -> Html {
    let qga: Result<PropertyString<QemuConfigAgent>, _> = serde_json::from_value(value.clone());

    match qga {
        Ok(qga) => {
            if !qga.enabled {
                return tr!("Disabled").into();
            }
            let mut parts = Vec::new();
            parts.push(tr!("Enabled"));

            if let Some(ty) = qga.ty {
                parts.push(ty.to_string());
            }
            if let Some(enabled) = qga.fstrim_cloned_disks {
                parts.push(format!("fstrim-cloned-disks: {}", render_boolean(enabled)));
            }
            if let Some(false) = qga.freeze_fs_on_backup {
                parts.push(format!("freeze-fs-on-backup: {}", render_boolean(false)));
            }
            parts.join(", ").into()
        }
        Err(err) => {
            log::error!("failed to parse qemu agent property: {err}");
            match value {
                Value::String(s) => s.into(),
                _ => value.into(),
            }
        }
    }
}

fn input_panel(name: String) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, _record| {
        let advanced = form_ctx.get_show_advanced();
        let enabled = form_ctx.read().get_field_checked(pspn(&name, "enabled"));
        let ffob_enabled = form_ctx
            .read()
            .get_field_checked(pspn(&name, "freeze-fs-on-backup"));

        let warning = |msg: String| {
            Container::new()
                .class("pwt-color-warning")
                .padding(1)
                .with_child(msg)
        };

        Column::new()
                        .class(pwt::css::FlexFit)
                        .with_child(
                            Checkbox::new()
                                .name(pspn(&name, "enabled"))
                                .box_label(tr!("Use QEMU Guest Agent")),
                        )
                        .with_child(
                            Checkbox::new()
                                .name(pspn(&name, "fstrim_cloned_disks"))
                                .box_label(tr!("Run guest-trim after a disk move or VM migration"))
                                .disabled(!enabled),
                        )
                        .with_child(
                            Checkbox::new()
                                .name(pspn(&name, "freeze-fs-on-backup"))
                                .box_label(tr!(
                                    "Freeze/thaw guest filesystems on backup for consistency"
                                ))
                                .disabled(!enabled),
                        )
                        .with_child(
                            crate::widgets::label_field(
                                tr!("Type"),
                                Combobox::from_key_value_pairs([
                                        ("virtio", "VirtIO"),
                                        ("isa", "ISA"),
                                ])
                                    .name(pspn(&name, "type"))
                                    .placeholder(tr!("Default") + " (VirtIO)")
                            ).class((!advanced).then(|| pwt::css::Display::None))
                            .padding_top(2)
                            .padding_bottom(1)
                        )
                        .with_optional_child((!ffob_enabled).then(|| warning(tr!(
                            "Freeze/thaw for guest filesystems disabled. This can lead to inconsistent disk backups."
                        ))))
                        .with_optional_child(enabled.then(|| warning(tr!(
                            "Make sure the QEMU Guest Agent is installed in the VM"
                        ))))
                        .into()
    })
}

pub fn qemu_agent_property() -> EditableProperty {
    let name = String::from("agent");
    EditableProperty::new(name.clone(), tr!("QEMU Guest Agent"))
        .advanced_checkbox(true)
        .required(true)
        .placeholder(format!("{} ({})", tr!("Default"), tr!("Disabled")))
        .renderer(renderer)
        .render_input_panel(input_panel(name.clone()))
        .load_hook(property_string_load_hook::<QemuConfigAgent>(&name))
        .submit_hook(property_string_submit_hook::<QemuConfigAgent>(&name, true))
}

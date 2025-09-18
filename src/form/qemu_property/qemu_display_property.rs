use std::rc::Rc;

use proxmox_human_byte::HumanByte;
use proxmox_schema::ApiType;
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Combobox, FormContext, Number};
use pwt::widget::{Column, Container};

use pve_api_types::{QemuConfigVga, QemuConfigVgaClipboard};

use crate::form::{
    flatten_property_string, format_qemu_display_type, property_string_from_parts, pspn,
    QemuDisplayTypeSelector,
};
use crate::widgets::{label_field, EditableProperty, RenderPropertyInputPanelFn};

fn input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, record: Rc<Value>| {
        //let show_efi_disk_hint =
        //    form_ctx.read().get_field_text("bios") == "ovmf" && record["efidisk0"].is_null();

        let vnc_hint = true; // fixme

        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .padding_bottom(1) // avoid scrollbar ?!
            .with_child(label_field(
                tr!("Graphic card"),
                QemuDisplayTypeSelector::new().name(pspn("vga", "type")),
            ))
            .with_child(label_field(
                tr!("Memory") + " (MiB)",
                Number::<u64>::new()
                    .name(pspn("vga", "memory"))
                    .min(4)
                    .max(512),
            ))
            //.with_child(label_field(
            //    tr!("Clipboard"),
            //    TODO
            //))
            .with_optional_child(vnc_hint.then(|| {
                hint(
                    tr!(
                    "You cannot use the default SPICE clipboard if the VNC clipboard is selected."
                ) + " " + &tr!("VNC clipboard requires spice-tools installed in the Guest-VM."),
                )
            }))
            .into()
    })
}

pub fn qemu_display_property() -> EditableProperty {
    EditableProperty::new("vga", tr!("Display"))
        .placeholder(tr!("Default"))
        .renderer(
            |_, v, _| match serde_json::from_value::<QemuConfigVga>(v.clone()) {
                Ok(vga) => {
                    let mut text = match vga.ty {
                        Some(ty) => format_qemu_display_type(&ty.to_string()),
                        None => tr!("Default"),
                    };

                    let mut inner = Vec::new();
                    if let Some(mb) = vga.memory {
                        let bytes = (mb as f64) * 1024.0 * 1024.0;
                        let memory = HumanByte::new_binary(bytes);
                        inner.push(format!("memory={memory}"));
                    };

                    if let Some(QemuConfigVgaClipboard::Vnc) = vga.clipboard {
                        inner.push(format!("clipboard=vnc"));
                    };
                    if !inner.is_empty() {
                        let inner = inner.join(", ");
                        text += &format!(" ({inner})");
                    }
                    text.into()
                }
                Err(_) => v.into(),
            },
        )
        .render_input_panel(input_panel())
        .load_hook(move |mut record: Value| {
            flatten_property_string(&mut record, "vga", &QemuConfigVga::API_SCHEMA)?;
            Ok(record)
        })
        .submit_hook({
            move |form_ctx: FormContext| {
                let mut record = form_ctx.get_submit_data();
                property_string_from_parts::<QemuConfigVga>(&mut record, "vga", true)?;
                record = delete_empty_values(&record, &["vga"], false);
                Ok(record)
            }
        })
}

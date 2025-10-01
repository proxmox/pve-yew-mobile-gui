use proxmox_schema::property_string::PropertyString;
use serde_json::{json, Value};

use pve_api_types::{QemuConfigSpiceEnhancements, QemuConfigVga, QemuConfigVgaType};

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Checkbox, Combobox};
use pwt::widget::{Column, Container};

use crate::form::{property_string_from_parts, property_string_load_hook};
use crate::widgets::{EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn};

fn input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |state: PropertyEditorState| {
        let form_ctx = state.form_ctx;
        let folder_sharing = form_ctx.read().get_field_checked("_foldersharing");

        let mut show_spice_hint = true;
        if let Some(Value::String(vga)) = state.record.get("vga") {
            if let Ok(vga) = proxmox_schema::property_string::parse::<QemuConfigVga>(vga) {
                match vga.ty {
                    Some(QemuConfigVgaType::Qxl)
                    | Some(QemuConfigVgaType::Qxl2)
                    | Some(QemuConfigVgaType::Qxl3)
                    | Some(QemuConfigVgaType::Qxl4) => show_spice_hint = false,
                    _ => {}
                }
            }
        }

        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        Column::new()
            .class(pwt::css::FlexFit)
            .padding_bottom(1) // avoid scrollbar ?!
            .gap(2)
            .with_child(
                Checkbox::new()
                    .name("_foldersharing")
                    .box_label(tr!("Folder Sharing")),
            )
            .with_child(crate::widgets::label_field(
                tr!("Video Streaming"),
                Combobox::new()
                    .name("_videostreaming")
                    .placeholder("off")
                    .with_item("all")
                    .with_item("filter"), true
            ))
            .with_optional_child(show_spice_hint.then(|| {
                hint(tr!(
                    "To use these features set the display to SPICE in the hardware settings of the VM."
                ))
            }))
            .with_optional_child(folder_sharing.then(|| {
                hint(tr!(
                    "Make sure the SPICE WebDav daemon is installed in the VM."
                ))
            }))
            .into()
    })
}

pub fn qemu_spice_enhancement_property() -> EditableProperty {
    let name = String::from("spice_enhancements");
    EditableProperty::new(name.clone(), tr!("Spice Enhancements"))
        .required(true)
        .placeholder(tr!("none"))
        .renderer(|_, v, _| {
            match serde_json::from_value::<Option<PropertyString<QemuConfigSpiceEnhancements>>>(
                v.clone(),
            ) {
                Ok(Some(data)) => {
                    let mut output = Vec::new();
                    if let Some(true) = data.foldersharing {
                        output.push(tr!("Folder Sharing"));
                    }
                    if let Some(videostreaming) = data.videostreaming {
                        output.push(tr!("Video Streaming") + ": " + &videostreaming.to_string());
                    }
                    if output.is_empty() {
                        return tr!("none").into();
                    } else {
                        return output.join(", ").into();
                    }
                }
                Ok(None::<_>) => tr!("none").into(),
                Err(err) => {
                    log::error!("qemu_spice_enhancement_property renderer: {err}");
                    v.into()
                }
            }
        })
        .render_input_panel(input_panel())
        .load_hook(property_string_load_hook::<QemuConfigSpiceEnhancements>(
            &name,
        ))
        .submit_hook({
            let name = name.clone();
            move |state: PropertyEditorState| {
                let form_data = state.get_submit_data();

                let mut value = json!({});

                let foldersharing_prop_name = "_foldersharing";
                if let Some(Value::Bool(true)) = form_data.get(foldersharing_prop_name) {
                    value[foldersharing_prop_name] = Value::Bool(true);
                }
                let videostreaming_prop_name = "_videostreaming";
                if let Some(Value::String(videostreaming)) = form_data.get(videostreaming_prop_name)
                {
                    if !videostreaming.is_empty() {
                        value[videostreaming_prop_name] = videostreaming.to_string().into();
                    }
                }

                property_string_from_parts::<QemuConfigSpiceEnhancements>(&mut value, &name, true)?;
                let value = delete_empty_values(&value, &[&name], false);
                Ok(value)
            }
        })
}

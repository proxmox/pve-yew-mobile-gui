use std::rc::Rc;

use proxmox_yew_comp::form::property_string_from_parts;
use serde_json::{json, Value};

use proxmox_schema::ApiType;

use pve_api_types::{QemuConfigSpiceEnhancements, QemuConfigVga, QemuConfigVgaType};

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Checkbox, Combobox, FormContext};
use pwt::widget::{Column, Container};

use crate::form::{property_string_load_hook, pspn};
use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};

fn input_panel(name: String) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, record: Rc<Value>| {
        let folder_sharing = form_ctx
            .read()
            .get_field_checked(pspn(&name, "foldersharing"));

        let mut show_spice_hint = true;
        if let Some(Value::String(vga)) = record.get("vga") {
            if let Ok(vga) = crate::form::parse_property_string::<QemuConfigVga>(vga) {
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
                    .name(pspn(&name, "foldersharing"))
                    .box_label(tr!("Folder Sharing")),
            )
            .with_child(crate::widgets::label_field(
                tr!("Video Streaming"),
                Combobox::new()
                    .name(pspn(&name, "videostreaming"))
                    .default("off")
                    .with_item("off")
                    .with_item("all")
                    .with_item("filter"),
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

pub fn qemu_spice_enhancement_property(name: impl Into<String>) -> EditableProperty {
    let name = name.into();
    EditableProperty::new(name.clone(), tr!("Spice Enhancements"))
        .required(true)
        .placeholder(tr!("none"))
        .renderer(|_, value, _| {
            if let Value::String(prop_str) = value {
                if let Ok(props) =
                    QemuConfigSpiceEnhancements::API_SCHEMA.parse_property_string(prop_str)
                {
                    let mut output = Vec::new();
                    if let Some(Value::Bool(true)) = props.get("foldersharing") {
                        output.push(tr!("Folder Sharing"));
                    }
                    if let Some(Value::String(videostreaming)) = props.get("videostreaming") {
                        if videostreaming == "all" || videostreaming == "filter" {
                            output.push(tr!("Video Streaming") + ": " + videostreaming);
                        }
                    }
                    if output.is_empty() {
                        return tr!("none").into();
                    } else {
                        return output.join(", ").into();
                    }
                }
            }
            value.into()
        })
        .render_input_panel(input_panel(name.clone()))
        .load_hook(property_string_load_hook::<QemuConfigSpiceEnhancements>(
            &name,
        ))
        .submit_hook({
            let name = name.clone();
            move |ctx: FormContext| {
                let form_data = ctx.get_submit_data();

                let mut value = json!({});

                let foldersharing_prop_name = pspn(&name, "foldersharing");
                if let Some(Value::Bool(true)) = form_data.get(&foldersharing_prop_name) {
                    value[foldersharing_prop_name] = Value::Bool(true);
                }
                let videostreaming_prop_name = pspn(&name, "videostreaming");
                if let Some(Value::String(videostreaming)) =
                    form_data.get(&videostreaming_prop_name)
                {
                    if videostreaming != "off" {
                        value[videostreaming_prop_name] = videostreaming.to_string().into();
                    }
                }

                property_string_from_parts::<QemuConfigSpiceEnhancements>(&mut value, &name, true);

                let value = delete_empty_values(&value, &[&name], false);
                Ok(value)
            }
        })
}

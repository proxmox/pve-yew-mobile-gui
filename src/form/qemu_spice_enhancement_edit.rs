use std::rc::Rc;

use proxmox_yew_comp::form::property_string_from_parts;
use pwt::props::SubmitCallback;
use serde_json::{json, Value};

use proxmox_schema::ApiType;

use pve_api_types::{QemuConfig, QemuConfigSpiceEnhancements, QemuConfigVga, QemuConfigVgaType};

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Checkbox, Combobox, FormContext};
use pwt::widget::{Column, Container};

use proxmox_yew_comp::http_put;

use crate::form::submit_property_string;
use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};

fn property_name(name: &str, prop: &str) -> String {
    format!("_{name}_{prop}")
}

fn input_panel(name: String) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, record: Rc<Value>| {
        let folder_sharing = form_ctx
            .read()
            .get_field_checked(property_name(&name, "foldersharing"));

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
                    .name(property_name(&name, "foldersharing"))
                    .box_label(tr!("Folder Sharing")),
            )
            .with_child(crate::widgets::label_field(
                tr!("Video Streaming"),
                Combobox::new()
                    .name(property_name(&name, "videostreaming"))
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

pub fn qemu_spice_enhancement_property(
    name: impl Into<String>,
    url: impl Into<String>,
) -> EditableProperty {
    let url = url.into();
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
        .loader(crate::form::load_property_string::<
            QemuConfig,
            QemuConfigSpiceEnhancements,
        >(&url, &name))
        .on_submit(Some(submit_property_string::<QemuConfigSpiceEnhancements>(
            &url, &name,
        )))
        .on_submit({
            let url = url.clone();
            SubmitCallback::new(move |ctx: FormContext| {
                let url = url.clone();
                let name = name.clone();
                async move {
                    let form_data = ctx.get_submit_data();

                    let mut value = json!({});

                    let foldersharing_prop_name = property_name(&name, "foldersharing");
                    if let Some(Value::Bool(true)) = form_data.get(&foldersharing_prop_name) {
                        value[foldersharing_prop_name] = Value::Bool(true);
                    }
                    let videostreaming_prop_name = property_name(&name, "videostreaming");
                    if let Some(Value::String(videostreaming)) =
                        form_data.get(&videostreaming_prop_name)
                    {
                        if videostreaming != "off" {
                            value[videostreaming_prop_name] = videostreaming.to_string().into();
                        }
                    }

                    property_string_from_parts::<QemuConfigSpiceEnhancements>(
                        &mut value, &name, true,
                    );

                    let value = delete_empty_values(&value, &[&name], false);
                    http_put(url.clone(), Some(value)).await
                }
            })
        })
}

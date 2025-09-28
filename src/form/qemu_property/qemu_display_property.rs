use std::rc::Rc;

use proxmox_human_byte::HumanByte;
use proxmox_schema::property_string::PropertyString;
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Combobox, FormContextObserver, Number};
use pwt::widget::{Column, Container};

use pve_api_types::{QemuConfigVga, QemuConfigVgaClipboard};
use yew::virtual_dom::VComp;

use crate::form::{
    flatten_property_string, format_qemu_display_type, property_string_from_parts,
    QemuDisplayTypeSelector,
};
use crate::widgets::{label_field, EditableProperty, PropertyEditorState};

fn renderer(_name: &str, value: &Value, _record: &Value) -> Html {
    let vga: Result<PropertyString<QemuConfigVga>, _> = serde_json::from_value(value.clone());
    match vga {
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
        Err(err) => {
            log::error!("failed to parse qemu vga property: {err}");
            match value {
                Value::String(s) => s.into(),
                _ => value.into(),
            }
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
struct StatefulPanel {
    state: PropertyEditorState,
}
struct StatefulPanelComp {
    serial_device_list: Rc<Vec<AttrValue>>,
    _form_ctx_observer: FormContextObserver,
}

impl Component for StatefulPanelComp {
    type Message = ();
    type Properties = StatefulPanel;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let record = &props.state.record;
        let mut serial_device_list = Vec::new();
        for i in 0..3 {
            let name = format!("serial{i}");
            if record[&name].as_str().is_some() {
                serial_device_list.push(AttrValue::from(name));
            }
        }
        // trigger re-draw on form context changes
        let _form_ctx_observer = props
            .state
            .form_ctx
            .add_listener(ctx.link().callback(|_| ()));
        Self {
            serial_device_list: Rc::new(serial_device_list),
            _form_ctx_observer,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let form_ctx = &props.state.form_ctx;

        let advanced = form_ctx.get_show_advanced();

        let vga_type = form_ctx.read().get_field_text("_type");
        let is_vnc = form_ctx.read().get_field_text("_clipboard") == "vnc";
        let has_gui = vga_type != "none" && !vga_type.starts_with("serial");

        let show_default_hint = !is_vnc && has_gui;
        let show_vnc_hint = is_vnc && has_gui;

        let memory_placeholder = match vga_type.as_str() {
            "cirrus" => 4.to_string(),
            "std" | "qxl" | "qxl2" | "qxl3" | "qxl4" | "vmware" => 16.to_string(),
            "virtio" | "virtio-gl" => 256.to_string(),
            _ if !has_gui => "N/A".into(),
            _ => tr!("Default"),
        };

        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        let vnc_hint =
            tr!("You cannot use the default SPICE clipboard if the VNC clipboard is selected.")
                + " "
                + &tr!("VNC clipboard requires spice-tools installed in the Guest-VM.");

        let migration_hint = tr!("You cannot live-migrate while using the VNC clipboard.");

        let default_hint = tr!("This option depends on your display type.")
            + " "
            + &tr!(
                "If the display type uses SPICE you are able to use the default SPICE clipboard."
            );

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .padding_bottom(1) // avoid scrollbar ?!
            .with_child(label_field(
                tr!("Graphic card"),
                QemuDisplayTypeSelector::new()
                    .name("_type")
                    .serial_device_list(Some(self.serial_device_list.clone())),
            ))
            .with_child(label_field(
                tr!("Memory") + " (MiB)",
                Number::<u64>::new()
                    .name("_memory")
                    .placeholder(memory_placeholder)
                    .disabled(!has_gui)
                    .min(4)
                    .max(512)
                    .step(4),
            ))
            .with_child(
                label_field(
                    tr!("Clipboard"),
                    Combobox::from_key_value_pairs([("", tr!("Default")), ("vnc", "VNC".into())])
                        .name("_clipboard")
                        .disabled(!has_gui),
                )
                .class((!advanced).then(|| pwt::css::Display::None)),
            )
            .with_optional_child(show_vnc_hint.then(|| hint(vnc_hint)))
            .with_optional_child(show_vnc_hint.then(|| hint(migration_hint)))
            .with_optional_child(show_default_hint.then(|| hint(default_hint)))
            .into()
    }
}

pub fn qemu_display_property() -> EditableProperty {
    EditableProperty::new("vga", tr!("Display"))
        .advanced_checkbox(true)
        .placeholder(tr!("Default"))
        .renderer(renderer)
        .render_input_panel(|state: PropertyEditorState| {
            let props = StatefulPanel { state };
            VComp::new::<StatefulPanelComp>(Rc::new(props), None).into()
        })
        .load_hook(move |mut record: Value| {
            flatten_property_string::<QemuConfigVga>(&mut record, "vga")?;
            Ok(record)
        })
        .submit_hook({
            move |state: PropertyEditorState| {
                let mut record = state.get_submit_data();
                property_string_from_parts::<QemuConfigVga>(&mut record, "vga", true)?;
                record = delete_empty_values(&record, &["vga"], false);
                Ok(record)
            }
        })
}

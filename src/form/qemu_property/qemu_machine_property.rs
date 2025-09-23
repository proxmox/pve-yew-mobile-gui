use std::rc::Rc;

use proxmox_schema::{ApiType, ObjectSchemaType, Schema};
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Combobox, FormContext, Hidden};
use pwt::widget::{Column, Container};

use pve_api_types::{QemuConfigMachine, QemuConfigOstype};

use crate::form::{
    flatten_property_string, property_string_from_parts, pspn, QemuMachineVersionSelector,
};
use crate::widgets::{label_field, EditableProperty, RenderPropertyInputPanelFn};
use crate::QemuMachineType;

fn ostype_is_windows(ostype: &QemuConfigOstype) -> bool {
    match ostype {
        QemuConfigOstype::Wxp
        | QemuConfigOstype::W2k
        | QemuConfigOstype::W2k3
        | QemuConfigOstype::W2k8
        | QemuConfigOstype::Wvista
        | QemuConfigOstype::Win7
        | QemuConfigOstype::Win8
        | QemuConfigOstype::Win10
        | QemuConfigOstype::Win11 => true,
        QemuConfigOstype::L24
        | QemuConfigOstype::L26
        | QemuConfigOstype::Solaris
        | QemuConfigOstype::Other => false,
    }
}

fn extract_machine_type(id: &str) -> QemuMachineType {
    if id == "q35" || id.starts_with("pc-q35-") {
        return QemuMachineType::Q35;
    }
    if id.is_empty() || id == "pc" || id.starts_with("pc-i440fx-") || id.starts_with("pc-") {
        return QemuMachineType::I440fx;
    }
    if id.starts_with("virt-") {
        return QemuMachineType::Virt;
    }
    log::error!("extract_machine_type failed: got '{id}'");
    QemuMachineType::I440fx
}

fn placeholder() -> String {
    tr!("Default") + &format!(" ({})", QemuMachineType::I440fx)
}

fn add_hidden_machine_properties(column: &mut Column, exclude: &[&str]) {
    // add unused machine property - we want to keep them!
    match QemuConfigMachine::API_SCHEMA {
        Schema::Object(object_schema) => {
            let props = object_schema.properties();
            for (part, _, _) in props {
                if !exclude.contains(part) {
                    column.add_child(Hidden::new().name(pspn("machine", part)));
                }
            }
        }
        _ => {
            log::error!(
                "add_hidden_machine_properties: internal error - got unsupported schema type"
            )
        }
    };
}

fn input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, record: Rc<Value>| {
        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        let advanced = form_ctx.get_show_advanced();

        let ostype: Option<QemuConfigOstype> =
            serde_json::from_value(record["ostype"].clone()).ok();
        let ostype = ostype.unwrap_or(QemuConfigOstype::Other);

        let extracted_type_prop_name = pspn("machine", "extracted-type");

        let machine_type = form_ctx
            .read()
            .get_field_value(extracted_type_prop_name.clone())
            .unwrap_or(Value::Null);
        let machine_type: QemuMachineType = serde_json::from_value(machine_type)
            .ok()
            .flatten()
            .unwrap_or(QemuMachineType::I440fx);

        let version_prop_name = pspn("machine", &format!("{machine_type}-version"));
        let show_version = match form_ctx.read().get_field_data(version_prop_name) {
            Some((Value::String(version), Ok(_), _)) => {
                if version.is_empty() || version == "pc" || version == "q35" {
                    advanced
                } else {
                    // avoid hiding a pinned version
                    true
                }
            }
            _ => true, // show field if we have errors
        };

        let add_version_selector = |column: &mut Column, ty| {
            let disabled = machine_type != ty;
            let name = format!("{ty}-version");
            let field = label_field(
                tr!("Version"),
                QemuMachineVersionSelector::new(ty)
                    .name(pspn("machine", &name))
                    .required(ostype_is_windows(&ostype))
                    .disabled(disabled)
                    .submit(false),
            )
            .class((disabled || !show_version).then(|| pwt::css::Display::None));

            column.add_child(field);
        };

        let mut column = Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .padding_bottom(1) // avoid scrollbar ?!
            .with_child(label_field(
                tr!("Type"),
                Combobox::new()
                    .name(extracted_type_prop_name.clone())
                    .required(true)
                    .submit(false)
                    .with_item("i440fx")
                    .with_item("q35")
                    .render_value(|v: &AttrValue| match v.as_str() {
                        "i440fx" => placeholder().into(),
                        "q35" => "Q35".into(),
                        _ => v.into(),
                    }),
            ));

        add_version_selector(&mut column, QemuMachineType::I440fx);
        add_version_selector(&mut column, QemuMachineType::Q35);
        add_version_selector(&mut column, QemuMachineType::Virt);

        let mut items = Vec::new();
        if machine_type == QemuMachineType::Q35 {
            items.push(("intel", tr!("Intel (AMD Compatible)")));
        }
        items.push(("virtio", tr!("VirtIO")));

        column.add_child(
            label_field(
                "vIOMMU",
                Combobox::from_key_value_pairs(items)
                    .name(pspn("machine", "viommu"))
                    .force_selection(true)
                    .placeholder(tr!("Default") + " (" + &tr!("None") + ")")
                    .render_value(|v: &AttrValue| {
                        match v.as_str() {
                            "intel" => tr!("Intel (AMD Compatible)"),
                            "virtio" => tr!("VirtIO"),
                            _ => v.to_string(),
                        }
                        .into()
                    }),
            )
            .class((!advanced).then(|| pwt::css::Display::None)),
        );

        column.add_optional_child(show_version.then(|| {
            hint(tr!(
                "Machine version change may affect hardware layout and settings in the guest OS."
            ))
        }));

        add_hidden_machine_properties(&mut column, &["type", "viommu"]);
        column.into()
    })
}

pub fn qemu_machine_property() -> EditableProperty {
    EditableProperty::new("machine", tr!("Machine"))
        .advanced_checkbox(true)
        .placeholder(placeholder())
        .renderer(move |_, v, record| {
            let ostype: Option<QemuConfigOstype> =
                serde_json::from_value(record["ostype"].clone()).ok();
            let ostype = ostype.unwrap_or(QemuConfigOstype::Other);
            match (v.as_str(), ostype_is_windows(&ostype)) {
                (None | Some("pc"), true) => "pc-i440fx-5.1".into(),
                (Some("q35"), true) => "pc-q35-5.1".into(),
                (Some(machine), _) => machine.into(),
                (None, _) => placeholder().into(),
            }
        })
        .render_input_panel(input_panel())
        .load_hook(move |mut record: Value| {
            flatten_property_string(&mut record, "machine", &QemuConfigMachine::API_SCHEMA)?;

            let machine_type_prop_name = pspn("machine", "type");
            let machine_type = record[&machine_type_prop_name].as_str().unwrap_or("");
            let machine_type = extract_machine_type(machine_type);

            let name = format!("{machine_type}-version");
            record[pspn("machine", &name)] = record[&machine_type_prop_name].take();

            let extracted_type_prop_name = pspn("machine", "extracted-type");
            record[extracted_type_prop_name] = machine_type.to_string().into();

            Ok(record)
        })
        .submit_hook({
            move |form_ctx: FormContext| {
                let mut data = form_ctx.get_submit_data();

                let machine_type_prop_name = pspn("machine", "type");
                let extracted_type_prop_name = pspn("machine", "extracted-type");

                let machine_type = form_ctx
                    .read()
                    .get_field_text(extracted_type_prop_name.clone());
                let name = pspn("machine", &format!("{machine_type}-version"));

                data[machine_type_prop_name] = form_ctx
                    .read()
                    .get_field_value(name.clone())
                    .unwrap_or(Value::Null);

                property_string_from_parts::<QemuConfigMachine>(&mut data, "machine", true)?;

                data = delete_empty_values(&data, &["machine"], false);
                Ok(data)
            }
        })
}

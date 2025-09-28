use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Combobox};
use pwt::widget::{Column, Container};

use pve_api_types::{QemuConfigMachine, QemuConfigOstype};

use crate::form::{
    flatten_property_string, property_string_add_missing_data, property_string_from_parts,
    QemuMachineVersionSelector,
};
use crate::widgets::{
    label_field, EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn,
};
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

fn input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |state: PropertyEditorState| {
        let form_ctx = state.form_ctx;
        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        let advanced = form_ctx.get_show_advanced();

        let ostype: Option<QemuConfigOstype> =
            serde_json::from_value(state.record["ostype"].clone()).ok();
        let ostype = ostype.unwrap_or(QemuConfigOstype::Other);

        let extracted_type_prop_name = "_extracted-type";

        let machine_type = form_ctx
            .read()
            .get_field_value(extracted_type_prop_name)
            .unwrap_or(Value::Null);
        let machine_type: QemuMachineType = serde_json::from_value(machine_type)
            .ok()
            .flatten()
            .unwrap_or(QemuMachineType::I440fx);

        let version_prop_name = format!("_{machine_type}-version");
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
            let name = format!("_{ty}-version");
            let field = label_field(
                tr!("Version"),
                QemuMachineVersionSelector::new(ty)
                    .name(name)
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
                    .name(extracted_type_prop_name)
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
                    .name("_viommu")
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
            flatten_property_string::<QemuConfigMachine>(&mut record, "machine")?;

            let machine_type = record["_type"].as_str().unwrap_or("");
            let machine_type = extract_machine_type(machine_type);

            let version_prop_name = format!("_{machine_type}-version");
            record[version_prop_name] = record["_type"].take();

            record["_extracted-type"] = machine_type.to_string().into();

            Ok(record)
        })
        .submit_hook({
            move |state: PropertyEditorState| {
                let form_ctx = state.form_ctx;
                let mut data = form_ctx.get_submit_data();

                let machine_type = form_ctx.read().get_field_text("_extracted-type");

                let version_prop_name = format!("{machine_type}-version");
                let mut version = form_ctx.read().get_field_text(version_prop_name);
                if version.is_empty() && machine_type == "q35" {
                    version = String::from("q35");
                }
                data["_type"] = version.into();

                property_string_add_missing_data::<QemuConfigMachine>(
                    &mut data,
                    &state.record,
                    &form_ctx,
                )?;
                property_string_from_parts::<QemuConfigMachine>(&mut data, "machine", true)?;

                data = delete_empty_values(&data, &["machine"], false);
                Ok(data)
            }
        })
}

use std::rc::Rc;

use proxmox_schema::ApiType;
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Combobox, FormContext};
use pwt::widget::{Column, Container};

use pve_api_types::{QemuConfigMachine, QemuConfigOstype};

use crate::form::{flatten_property_string, pspn};
use crate::widgets::{label_field, EditableProperty, RenderPropertyInputPanelFn};

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

fn input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, _record: Rc<Value>| {
        let show_hint = false;

        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .padding_bottom(1) // avoid scrollbar ?!
            .with_child(label_field(
                tr!("Type"),
                Combobox::new()
                    .name(pspn("machine", "extracted_type"))
                    .placeholder(tr!("Default") + " (i440fx)"),
            ))
            .with_child(label_field(
                tr!("Version"),
                Combobox::new()
                    .name(pspn("machine", "version"))
                    .placeholder(tr!("Latest")),
            ))
            .with_child(hint(tr!(
                "Machine version change may affect hardware layout and settings in the guest OS."
            )))
            .into()
    })
}

pub fn qemu_machine_property() -> EditableProperty {
    let placeholder = tr!("Default") + " (i440fx)";
    EditableProperty::new("machine", tr!("Machine"))
        .placeholder(placeholder.clone())
        .renderer(move |_, v, record| {
            let ostype: Option<QemuConfigOstype> =
                serde_json::from_value(record["ostype"].clone()).ok();
            let ostype = ostype.unwrap_or(QemuConfigOstype::Other);
            match (v.as_str(), ostype_is_windows(&ostype)) {
                (None | Some("pc"), true) => "pc-i440fx-5.1".into(),
                (Some("q35"), true) => "pc-q35-5.1".into(),
                (Some(machine), _) => machine.into(),
                (None, _) => placeholder.clone().into(),
            }
        })
        .render_input_panel(input_panel())
        .load_hook(move |mut record: Value| {
            flatten_property_string(&mut record, "machine", &QemuConfigMachine::API_SCHEMA)?;
            Ok(record)
        })
        .submit_hook({
            move |form_ctx: FormContext| {
                let mut data = form_ctx.get_submit_data();
                data = delete_empty_values(&data, &["machine"], false);
                Ok(data)
            }
        })
}

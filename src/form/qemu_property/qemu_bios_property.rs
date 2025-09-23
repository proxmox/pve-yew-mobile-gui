use std::rc::Rc;

use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Combobox, FormContext};
use pwt::widget::{Column, Container};

use pve_api_types::QemuConfigBios;

use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};

fn input_panel(name: String) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |form_ctx: FormContext, record: Rc<Value>| {
        let show_efi_disk_hint =
            form_ctx.read().get_field_text("bios") == "ovmf" && record["efidisk0"].is_null();

        let hint = |msg: String| Container::new().class("pwt-color-warning").with_child(msg);

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .padding_bottom(1) // avoid scrollbar ?!
            .with_child(
                Combobox::from_key_value_pairs([
                        ("ovmf", "OVMF (UEFI)"),
                        ("seabios", "SeaBIOS"),
                ])
                    .name(name.clone())
                    .submit_empty(true)
                    .placeholder("SeaBIOS")
            )
            .with_optional_child(show_efi_disk_hint.then(|| {
                hint(tr!(
                    "You need to add an EFI disk for storing the EFI settings. See the online help for details."
                ))
            }))
            .into()
    })
}

pub fn qemu_bios_property() -> EditableProperty {
    let name = String::from("bios");
    EditableProperty::new(name.clone(), "BIOS")
        .placeholder(tr!("Default") + " (SeaBIOS)")
        .renderer(
            |_, v, _| match serde_json::from_value::<QemuConfigBios>(v.clone()) {
                Ok(bios) => match bios {
                    QemuConfigBios::Seabios => "SeaBIOS".into(),
                    QemuConfigBios::Ovmf => "OVMF (UEFI)".into(),
                },
                Err(_) => v.into(),
            },
        )
        .render_input_panel(input_panel(name.clone()))
        .submit_hook({
            let name = name.clone();
            move |form_ctx: FormContext| {
                let mut data = form_ctx.get_submit_data();
                data = delete_empty_values(&data, &[&name], false);
                Ok(data)
            }
        })
}

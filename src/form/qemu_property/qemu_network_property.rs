use serde_json::Value;

use pve_api_types::QemuConfigNet;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Checkbox, Combobox, Field, FormContext};
use pwt::widget::{Column, Row};

use crate::form::{
    flatten_property_string, property_string_from_parts, pspn, PveNetworkSelector, PveVlanField,
};
use crate::widgets::{label_field, EditableProperty, RenderPropertyInputPanelFn};

fn input_panel(name: &str, node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    let name = name.to_string();
    RenderPropertyInputPanelFn::new(move |_form_ctx: FormContext, _| {
        //let advanced = form_ctx.get_show_advanced();

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("Bridge"),
                PveNetworkSelector::new()
                    .node(node.clone())
                    .name(pspn(&name, "bridge"))
                    .required(true),
            ))
            .with_child(label_field(
                tr!("Model"),
                Combobox::from_key_value_pairs([
                    ("e1000", String::from("Intel E1000")),
                    ("e1000e", String::from("Intel E1000E")),
                    (
                        "virtio",
                        String::from("VirtIO (") + &tr!("paravirtualized") + ")",
                    ),
                    ("rtl8139", String::from("Realtek RTL8139")),
                    ("vmxnet3", String::from("VMware vmxnet3")),
                ])
                .name(pspn(&name, "model"))
                .required(true),
            ))
            .with_child(label_field(
                PveVlanField::get_std_label(),
                PveVlanField::new().name(pspn(&name, "tag")),
            ))
            .with_child(label_field(
                tr!("MAC address"),
                Field::new()
                    .name(pspn(&name, "macaddr"))
                    .placeholder("auto"),
            ))
            .with_child(
                Row::new()
                    .gap(2)
                    .with_child(label_field(
                        tr!("Firewall"),
                        Checkbox::new().name(pspn(&name, "firewall")),
                    ))
                    // .with_flex_spacer()
                    .with_child(label_field(
                        tr!("Disconnect"),
                        Checkbox::new().name(pspn(&name, "link_down")),
                    )),
            )
            .into()
    })
}

pub fn qemu_network_property(name: &str, node: Option<AttrValue>) -> EditableProperty {
    let name = name.to_string();
    EditableProperty::new(name.clone(), tr!("Network Device") + &format!(" ({name})"))
        .advanced_checkbox(true)
        .render_input_panel(input_panel(&name, node.clone()))
        .submit_hook({
            let name = name.clone();
            move |form_ctx: FormContext| {
                let mut data = form_ctx.get_submit_data();
                property_string_from_parts::<QemuConfigNet>(&mut data, &name, true)?;
                data = delete_empty_values(&data, &[&name], false);
                Ok(data)
            }
        })
        .load_hook({
            let name = name.clone();
            move |mut record: Value| {
                flatten_property_string::<QemuConfigNet>(&mut record, &name)?;
                Ok(record)
            }
        })
}

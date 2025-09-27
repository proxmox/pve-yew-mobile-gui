use anyhow::bail;
use serde_json::Value;

use pve_api_types::QemuConfigNet;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Checkbox, Combobox, Field, Number};
use pwt::widget::{Column, Row};

use crate::form::{
    flatten_property_string, property_string_add_missing_data, property_string_from_parts, pspn,
    PveNetworkSelector, PveVlanField,
};
use crate::widgets::{
    label_field, EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn,
};

fn input_panel(name: &str, node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    let name = name.to_string();
    RenderPropertyInputPanelFn::new(move |_| {
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
                PveVlanField::new()
                    .name(pspn(&name, "tag"))
                    .submit_empty(true),
            ))
            .with_child(label_field(
                tr!("MAC address"),
                Field::new()
                    .name(pspn(&name, "macaddr"))
                    .placeholder("auto")
                    .submit_empty(true),
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
        .render_input_panel(input_panel(&name, node.clone()))
        .submit_hook({
            let name = name.clone();
            move |state: PropertyEditorState| {
                let mut data = state.get_submit_data();
                property_string_add_missing_data::<QemuConfigNet>(&mut data, &state.record, &name)?;
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

fn mtu_input_panel(name: &str) -> RenderPropertyInputPanelFn {
    let name = name.to_string();
    RenderPropertyInputPanelFn::new(move |_| {
        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("MTU"),
                Number::<u16>::new()
                    .name(pspn(&name, "mtu"))
                    .placeholder("Same as bridge")
                    .submit_empty(true)
                    .min(1)
                    .max(65520)
                    .validate(|val: &u16| {
                        if *val >= 576 || *val == 1 {
                            return Ok(());
                        }
                        bail!("MTU needs to be >= 576 or 1 to inherit the MTU from the underlying bridge.");
                    }),
            ))
            .with_child(label_field(
                tr!("Rate limit") + " (MB/s)",
                Number::<f64>::new()
                    .name(pspn(&name, "rate"))
                    .placeholder(tr!("unlimited"))
                    .submit_empty(true)
                    .min(0.0)
                    .max(10.0 * 1024.0)
            ))
            .with_child(label_field(
                tr!("Multiqueue"),
                Number::<u8>::new()
                    .name(pspn(&name, "queues"))
                    .submit_empty(true)
                    .min(1)
                    .max(64)
            )).into()
    })
}

pub fn qemu_network_mtu_property(name: &str, node: Option<AttrValue>) -> EditableProperty {
    let mut property = qemu_network_property(name, node).render_input_panel(mtu_input_panel(&name));
    property.title = format!("MTU, {}, Multiqueue ({name})", tr!("Rate limit")).into();
    property
}

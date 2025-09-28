use anyhow::{bail, Error};
use serde_json::Value;

use pve_api_types::QemuConfigNet;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Checkbox, Combobox, Field, Number};
use pwt::widget::{Column, Row};

use crate::form::{
    flatten_property_string, flatten_property_string_new, property_string_add_missing_data,
    property_string_add_missing_data_new, property_string_from_parts,
    property_string_from_parts_new, PveNetworkSelector, PveVlanField,
};
use crate::widgets::{
    label_field, EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn,
};

fn input_panel(node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_| {
        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("Bridge"),
                PveNetworkSelector::new()
                    .node(node.clone())
                    .name("_bridge")
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
                .name("_model")
                .required(true)
                .default("virtio"),
            ))
            .with_child(label_field(
                PveVlanField::get_std_label(),
                PveVlanField::new().name("_tag").submit_empty(true),
            ))
            .with_child(label_field(
                tr!("MAC address"),
                Field::new()
                    .name("_macaddr")
                    .placeholder("auto")
                    .submit_empty(true),
            ))
            .with_child(
                Row::new()
                    .gap(2)
                    .with_child(label_field(
                        tr!("Firewall"),
                        Checkbox::new().name("_firewall"),
                    ))
                    .with_child(label_field(
                        tr!("Disconnect"),
                        Checkbox::new().name("_link_down"),
                    )),
            )
            .into()
    })
}

fn find_free_network(record: &Value) -> Result<String, Error> {
    if let Some(map) = record.as_object() {
        for i in 0..16 {
            let name = format!("net{i}");
            if !map.contains_key(&name) {
                return Ok(name);
            }
        }
        bail!(tr!("All network devices in use."));
    } else {
        Ok("net0".into())
    }
}

pub fn qemu_network_property(name: Option<String>, node: Option<AttrValue>) -> EditableProperty {
    let mut title = tr!("Network Device");
    if let Some(name) = name.as_deref() {
        title = title + " (" + name + ")";
    }
    EditableProperty::new(
        name.as_ref().map(|s| s.clone()).unwrap_or(String::new()),
        title,
    )
    .render_input_panel(input_panel(node.clone()))
    .submit_hook({
        let name = name.clone();
        move |state: PropertyEditorState| {
            let mut data = state.get_submit_data();
            let network = find_free_network(&state.record)?;
            let name = name.clone().unwrap_or(network);
            property_string_add_missing_data_new::<QemuConfigNet>(&mut data, &state.record)?;
            property_string_from_parts_new::<QemuConfigNet>(&mut data, &name, true)?;
            data = delete_empty_values(&data, &[&name], false);
            Ok(data)
        }
    })
    .load_hook({
        let name = name.clone();
        move |mut record: Value| {
            if let Some(name) = name.as_deref() {
                flatten_property_string_new::<QemuConfigNet>(&mut record, name)?;
            } else {
                let _ = find_free_network(&record)?; // test early
            }
            Ok(record)
        }
    })
}

fn mtu_input_panel() -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_| {
        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("MTU"),
                Number::<u16>::new()
                    .name("_mtu")
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
                    .name("_rate")
                    .placeholder(tr!("unlimited"))
                    .submit_empty(true)
                    .min(0.0)
                    .max(10.0 * 1024.0)
            ))
            .with_child(label_field(
                tr!("Multiqueue"),
                Number::<u8>::new()
                    .name("_queues")
                    .submit_empty(true)
                    .min(1)
                    .max(64)
            )).into()
    })
}

pub fn qemu_network_mtu_property(
    name: Option<String>,
    node: Option<AttrValue>,
) -> EditableProperty {
    let mut property =
        qemu_network_property(name.clone(), node).render_input_panel(mtu_input_panel());

    let mut title = format!("MTU, {}, Multiqueue", tr!("Rate limit"));
    if let Some(name) = name.as_deref() {
        title = title + " (" + name + ")";
    }
    property.title = title.into();
    property
}

use anyhow::bail;
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::{form::delete_empty_values, Column};

use pve_api_types::{PveQmIde, QemuConfigSata, QemuConfigScsi, QemuConfigVirtio};

use crate::{
    form::{
        flatten_property_string, property_string_add_missing_data, property_string_from_parts,
        QemuControllerSelector,
    },
    widgets::{label_field, EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn},
};

fn input_panel(node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_| {
        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child("TEST")
            .into()
    })
}

pub fn qemu_disk_property(name: Option<String>, node: Option<AttrValue>) -> EditableProperty {
    let mut title = tr!("Hard Disk");
    if let Some(name) = name.as_deref() {
        title = title + " (" + name + ")";
    }
    EditableProperty::new(
        name.as_ref().map(|s| s.clone()).unwrap_or(String::new()),
        title,
    )
    .render_input_panel(input_panel(node.clone()))
}

fn cdrom_input_panel(node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_| {
        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(label_field(
                tr!("Bus/Device"),
                QemuControllerSelector::new().name("_device_").submit(false),
            ))
            // .with_child("CDROM TEST")
            .into()
    })
}

pub fn qemu_cdrom_property(name: Option<String>, node: Option<AttrValue>) -> EditableProperty {
    let mut title = tr!("CD/DVD Drive");
    if let Some(name) = name.as_deref() {
        title = title + " (" + name + ")";
    }
    EditableProperty::new(
        name.as_ref().map(|s| s.clone()).unwrap_or(String::new()),
        title,
    )
    .render_input_panel(cdrom_input_panel(node.clone()))
    .load_hook({
        let name = name.clone();

        move |mut record: Value| {
            if let Some(name) = &name {
                if name.starts_with("ide") {
                    flatten_property_string::<PveQmIde>(&mut record, name)?;
                } else if name.starts_with("sata") {
                    flatten_property_string::<QemuConfigSata>(&mut record, name)?;
                } else if name.starts_with("scsi") {
                    flatten_property_string::<QemuConfigScsi>(&mut record, name)?;
                } else if name.starts_with("virtio") {
                    flatten_property_string::<QemuConfigVirtio>(&mut record, name)?;
                } else {
                    bail!("qemu_cdrom_property: unsupported device type '{name}'");
                }
            }

            record["_device_"] = name.clone().into();

            Ok(record)
        }
    })
    .submit_hook({
        //let name = name.clone();

        move |state: PropertyEditorState| {
            let form_ctx = state.form_ctx;
            let mut data = form_ctx.get_submit_data();

            let device = form_ctx.read().get_field_text("_device_");

            if device.starts_with("ide") {
                property_string_add_missing_data::<PveQmIde>(&mut data, &state.record, &form_ctx)?;
                property_string_from_parts::<PveQmIde>(&mut data, &device, true)?;
            } else if device.starts_with("sata") {
                property_string_add_missing_data::<QemuConfigSata>(
                    &mut data,
                    &state.record,
                    &form_ctx,
                )?;
                property_string_from_parts::<QemuConfigSata>(&mut data, &device, true)?;
            } else if device.starts_with("scsi") {
                property_string_add_missing_data::<QemuConfigScsi>(
                    &mut data,
                    &state.record,
                    &form_ctx,
                )?;
                property_string_from_parts::<QemuConfigScsi>(&mut data, &device, true)?;
            } else if device.starts_with("virtio") {
                property_string_add_missing_data::<QemuConfigVirtio>(
                    &mut data,
                    &state.record,
                    &form_ctx,
                )?;
                property_string_from_parts::<QemuConfigVirtio>(&mut data, &device, true)?;
            } else {
                bail!("qemu_cdrom_property: unsupported device type '{device}'");
            }
            data = delete_empty_values(&data, &[&device], false);

            Ok(data)
        }
    })
}

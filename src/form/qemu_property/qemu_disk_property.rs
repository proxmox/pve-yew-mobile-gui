use anyhow::bail;
use pwt::widget::form::RadioButton;
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::{form::delete_empty_values, Column};

use pve_api_types::{PveQmIde, QemuConfigSata, QemuConfigScsi, QemuConfigVirtio, StorageContent};

const MEDIA_TYPE: &'static str = "_media_type_";
const BUS_DEVICE: &'static str = "_device_";
const IMAGE_STORAGE: &'static str = "_storage_";

use crate::form::pve_storage_content_selector::PveStorageContentSelector;
use crate::form::PveStorageSelector;
use crate::form::{
    flatten_property_string, property_string_add_missing_data, property_string_from_parts,
    QemuControllerSelector,
};

use crate::widgets::{
    label_field, EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn,
};

fn input_panel(_node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
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

fn cdrom_input_panel(name: Option<String>, node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    let is_create = name.is_none();
    RenderPropertyInputPanelFn::new(move |state: PropertyEditorState| {
        let form_ctx = state.form_ctx;
        let media_type = form_ctx.read().get_field_text(MEDIA_TYPE);
        let image_storage = form_ctx.read().get_field_text(IMAGE_STORAGE);

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_optional_child(is_create.then(|| {
                label_field(
                    tr!("Bus/Device"),
                    QemuControllerSelector::new().name(BUS_DEVICE).submit(false),
                    true,
                )
            }))
            .with_child(
                RadioButton::new("iso")
                    .default(true)
                    .box_label(tr!("Use CD/DVD disc image file (iso)"))
                    .name(MEDIA_TYPE)
                    .submit(false),
            )
            .with_child(label_field(
                tr!("Storage"),
                PveStorageSelector::new(node.clone())
                    .name(IMAGE_STORAGE)
                    .mobile(true),
                media_type == "iso",
            ))
            .with_child(label_field(
                tr!("ISO image"),
                PveStorageContentSelector::new()
                    .name("_file")
                    .node(node.clone())
                    .storage(image_storage.clone())
                    .content_filter(StorageContent::Iso),
                media_type == "iso",
            ))
            .with_child(
                RadioButton::new("cdrom")
                    .box_label(tr!("Use physical CD/DVD Drive"))
                    .name(MEDIA_TYPE)
                    .submit(false),
            )
            .with_child(
                RadioButton::new("none")
                    .box_label(tr!("Do not use any media"))
                    .name(MEDIA_TYPE)
                    .submit(false),
            )
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
    .render_input_panel(cdrom_input_panel(name.clone(), node.clone()))
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

            record[BUS_DEVICE] = name.clone().into();

            match record["_file"].as_str() {
                Some("cdrom") => record[MEDIA_TYPE] = "cdrom".into(),
                Some("none") => record[MEDIA_TYPE] = "none".into(),
                Some(volid) => {
                    if let Some((storage, _rest)) = volid.split_once(':') {
                        record[IMAGE_STORAGE] = storage.into();
                    }
                }
                _ => {}
            }

            Ok(record)
        }
    })
    .submit_hook({
        //let name = name.clone();

        move |state: PropertyEditorState| {
            let form_ctx = state.form_ctx;
            let mut data = form_ctx.get_submit_data();

            let device = match &name {
                Some(name) => name.clone(),
                None::<_> => form_ctx.read().get_field_text(BUS_DEVICE),
            };

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

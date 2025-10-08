use std::collections::HashSet;

use anyhow::{bail, Error};
use proxmox_schema::property_string::PropertyString;
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Number, RadioButton};
use pwt::widget::{Column, Container, Row};

use pve_api_types::{
    PveQmIde, QemuConfigSata, QemuConfigScsi, QemuConfigScsiArray, QemuConfigUnused,
    QemuConfigVirtio, StorageContent,
};

const MEDIA_TYPE: &'static str = "_media_type_";
const BUS_DEVICE: &'static str = "_device_";
const IMAGE_STORAGE: &'static str = "_storage_";
//const STORAGE_INFO: &'static str = "_storage_info_";
const DISK_SIZE: &'static str = "_disk_size_";

const FILE_PN: &'static str = "_file";

use crate::form::pve_storage_content_selector::PveStorageContentSelector;
use crate::form::{
    flatten_property_string, property_string_add_missing_data, property_string_from_parts,
    QemuCacheTypeSelector, QemuControllerSelector,
};
use crate::form::{parse_qemu_controller_name, PveStorageSelector};

use crate::widgets::{
    label_field, EditableProperty, PropertyEditorState, RenderPropertyInputPanelFn,
};

fn disk_input_panel(name: Option<String>, node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    let is_create = name.is_none();
    RenderPropertyInputPanelFn::new(move |state: PropertyEditorState| {
        let used_devices = extract_used_devices(&state.record);

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_optional_child(is_create.then(|| {
                label_field(
                    tr!("Bus/Device"),
                    QemuControllerSelector::new()
                        .name(BUS_DEVICE)
                        .submit(false)
                        .exclude_devices(used_devices),
                    true,
                )
            }))
            .with_optional_child((!is_create).then(|| {
                let file_text = match state.record.get(FILE_PN) {
                    Some(Value::String(file)) => file.clone(),
                    _ => String::new(),
                };
                let size_text = match state.record.get("_size") {
                    Some(Value::String(s)) => s.clone(),
                    _ => "-".into(),
                };
                Row::new()
                    .gap(1)
                    .with_child(Container::new().with_child(file_text))
                    .with_flex_spacer()
                    .with_child(Container::new().with_child(size_text))
            }))
            .with_child(label_field(
                tr!("Cache"),
                QemuCacheTypeSelector::new().name("_cache"),
                true,
            ))
            .with_optional_child(is_create.then(|| {
                label_field(
                    tr!("Storage"),
                    PveStorageSelector::new(node.clone())
                        .name(IMAGE_STORAGE)
                        .submit(false)
                        .required(true)
                        .content_types(Some(vec![StorageContent::Images]))
                        .mobile(true),
                    true,
                )
            }))
            .with_optional_child(is_create.then(|| {
                label_field(
                    tr!("Disk size") + " (GiB)",
                    Number::<f64>::new()
                        .name(DISK_SIZE)
                        .submit(false)
                        .required(true)
                        .min(0.001)
                        .max(128.0 * 1024.0)
                        .default(32.0),
                    true,
                )
            }))
            /*
            // fixme: boolean in property strings does not work currently
            .with_child(
                Row::new()
                    .gap(2)
                    .style("flex-wrap", "wrap")
                    .with_child(
                        label_field(
                            tr!("Discard"),
                            Checkbox::new().name("_discard").default(true),
                            true,
                        )
                        .class(pwt::css::JustifyContent::SpaceBetween),
                    )
                    .with_child(
                        label_field(tr!("IO thread"), Checkbox::new().name("_iothread"), true)
                            .class(pwt::css::JustifyContent::SpaceBetween),
                    )
                    .with_child(
                        label_field(tr!("SSD emulation"), Checkbox::new().name("_ssd"), true)
                            .class(pwt::css::JustifyContent::SpaceBetween),
                    )
                    .with_child(
                        label_field(tr!("Backup"), Checkbox::new().name("_backup"), true)
                            .class(pwt::css::JustifyContent::SpaceBetween),
                    )
                    .with_child(
                        label_field(
                            tr!("Skip replication"),
                            Checkbox::new().name("_noreplicate"),
                            true,
                        )
                        .class(pwt::css::JustifyContent::SpaceBetween),
                    )
                    .with_child(
                        label_field(tr!("Read-only"), Checkbox::new().name("_readOnly"), true)
                            .class(pwt::css::JustifyContent::SpaceBetween),
                    ),
            )
            */
            .into()
    })
}

pub fn qemu_disk_property(name: Option<String>, node: Option<AttrValue>) -> EditableProperty {
    let mut title = tr!("Hard Disk");
    if let Some(name) = name.as_deref() {
        title = title + " (" + name + ")";
    }

    EditableProperty::new(name.clone(), title)
        .render_input_panel(disk_input_panel(name.clone(), node.clone()))
        .load_hook({
            let name = name.clone();

            move |mut record: Value| {
                if let Some(name) = &name {
                    flatten_device_data(&mut record, name)?;
                    record[BUS_DEVICE] = name.clone().into();
                } else {
                    let used_devices = extract_used_devices(&record);
                    let default_device = first_unused_scsi_device(&used_devices);
                    record[BUS_DEVICE] = default_device.clone().into();
                }

                Ok(record)
            }
        })
        .submit_hook({
            let name = name.clone();

            move |state: PropertyEditorState| {
                let form_ctx = &state.form_ctx;
                let mut data = form_ctx.get_submit_data();
                let is_create = name.is_none();

                let device = match &name {
                    Some(name) => name.clone(),
                    None::<_> => form_ctx.read().get_field_text(BUS_DEVICE),
                };

                if is_create {
                    let image_storage = form_ctx.read().get_field_text(IMAGE_STORAGE);
                    let image_size = match form_ctx.read().get_last_valid_value(DISK_SIZE) {
                        Some(Value::Number(size)) => size.as_f64().unwrap(),
                        _ => bail!("got invalid disk size"),
                    };
                    let image = format!("{image_storage}:{image_size}");
                    data[FILE_PN] = image.into();
                }

                let data = assemble_device_data(&state, &mut data, &device)?;
                Ok(data)
            }
        })
}

fn add_unused_disk_panel(name: String, _node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |state: PropertyEditorState| {
        let used_devices = extract_used_devices(&state.record);

        let disk_image = state
            .record
            .get(&name)
            .map(|v| v.as_str())
            .flatten()
            .unwrap_or("unknown");

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_child(Container::new().with_child(disk_image))
            .with_child(label_field(
                tr!("Bus/Device"),
                QemuControllerSelector::new()
                    .name(BUS_DEVICE)
                    .submit(false)
                    .exclude_devices(used_devices),
                true,
            ))
            .into()
    })
}

pub fn qemu_unused_disk_property(name: &str, node: Option<AttrValue>) -> EditableProperty {
    let title = tr!("Unused Disk");

    EditableProperty::new(name.to_string(), title)
        .render_input_panel(add_unused_disk_panel(name.to_string(), node.clone()))
        .load_hook({
            // let name = name.to_string();
            move |mut record: Value| {
                let used_devices = extract_used_devices(&record);
                let default_device = first_unused_scsi_device(&used_devices);
                record[BUS_DEVICE] = default_device.clone().into();
                Ok(record)
            }
        })
        .submit_hook({
            let name = name.to_string();

            move |state: PropertyEditorState| {
                let form_ctx = &state.form_ctx;
                let mut data = form_ctx.get_submit_data();

                let device = form_ctx.read().get_field_text(BUS_DEVICE);
                let unused: PropertyString<QemuConfigUnused> =
                    serde_json::from_value(state.record[&name].clone())?;

                data[FILE_PN] = unused.file.clone().into();

                let data = assemble_device_data(&state, &mut data, &device)?;
                Ok(data)
            }
        })
}

fn extract_used_devices(record: &Value) -> HashSet<String> {
    let mut list = HashSet::new();
    if let Some(map) = record.as_object() {
        for key in map.keys() {
            if let Ok(_) = parse_qemu_controller_name(key) {
                list.insert(key.to_string());
            }
        }
    }
    list
}

fn first_unused_scsi_device(used_devices: &HashSet<String>) -> Option<String> {
    for n in 0..QemuConfigScsiArray::MAX {
        let name = format!("scsi{n}");
        if !used_devices.contains(&name) {
            return Some(name);
        }
    }
    None
}

fn cdrom_input_panel(name: Option<String>, node: Option<AttrValue>) -> RenderPropertyInputPanelFn {
    let is_create = name.is_none();
    RenderPropertyInputPanelFn::new(move |state: PropertyEditorState| {
        let form_ctx = state.form_ctx;
        let media_type = form_ctx.read().get_field_text(MEDIA_TYPE);
        let image_storage = form_ctx.read().get_field_text(IMAGE_STORAGE);

        let used_devices = extract_used_devices(&state.record);

        Column::new()
            .class(pwt::css::FlexFit)
            .gap(2)
            .with_optional_child(is_create.then(|| {
                label_field(
                    tr!("Bus/Device"),
                    QemuControllerSelector::new()
                        .name(BUS_DEVICE)
                        .submit(false)
                        .exclude_devices(used_devices),
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
                    .submit(false)
                    .required(true)
                    .autoselect(true)
                    .mobile(true),
                media_type == "iso",
            ))
            .with_child(label_field(
                tr!("ISO image"),
                PveStorageContentSelector::new()
                    .name(FILE_PN)
                    .required(true)
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
    EditableProperty::new(name.clone(), title)
        .render_input_panel(cdrom_input_panel(name.clone(), node.clone()))
        .load_hook({
            let name = name.clone();

            move |mut record: Value| {
                if let Some(name) = &name {
                    flatten_device_data(&mut record, name)?;
                }

                record[BUS_DEVICE] = name.clone().into();

                match record["_file"].as_str() {
                    Some("cdrom") => {
                        record[MEDIA_TYPE] = "cdrom".into();
                        record[FILE_PN] = Value::Null;
                    }
                    Some("none") => {
                        record[MEDIA_TYPE] = "none".into();
                        record[FILE_PN] = Value::Null;
                    }
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
            let name = name.clone();

            move |state: PropertyEditorState| {
                let form_ctx = &state.form_ctx;
                let mut data = form_ctx.get_submit_data();

                let device = match &name {
                    Some(name) => name.clone(),
                    None::<_> => form_ctx.read().get_field_text(BUS_DEVICE),
                };

                let media_type = form_ctx.read().get_field_text(MEDIA_TYPE);

                match media_type.as_str() {
                    "cdrom" => data[FILE_PN] = "cdrom".into(),
                    "none" => data[FILE_PN] = "none".into(),
                    _ => {}
                };

                data["_media"] = "cdrom".into();

                let data = assemble_device_data(&state, &mut data, &device)?;

                Ok(data)
            }
        })
        .on_change(|state: PropertyEditorState| {
            let form_ctx = state.form_ctx;
            let image_storage = form_ctx.read().get_field_text(IMAGE_STORAGE);
            let file = form_ctx.read().get_field_text(FILE_PN);
            if !image_storage.is_empty() {
                if !file.starts_with(&(image_storage + ":")) {
                    form_ctx.write().set_field_value(FILE_PN, "".into());
                }
            }
        })
}

fn flatten_device_data(record: &mut Value, name: &str) -> Result<(), Error> {
    if name.starts_with("ide") {
        flatten_property_string::<PveQmIde>(record, name)?;
    } else if name.starts_with("sata") {
        flatten_property_string::<QemuConfigSata>(record, name)?;
    } else if name.starts_with("scsi") {
        flatten_property_string::<QemuConfigScsi>(record, name)?;
    } else if name.starts_with("virtio") {
        flatten_property_string::<QemuConfigVirtio>(record, name)?;
    } else {
        bail!("flatten_device_data: unsupported device type '{name}'");
    }
    Ok(())
}

fn assemble_device_data(
    state: &PropertyEditorState,
    data: &mut Value,
    device: &str,
) -> Result<Value, Error> {
    let form_ctx = &state.form_ctx;
    if device.starts_with("ide") {
        property_string_add_missing_data::<PveQmIde>(data, &state.record, &form_ctx)?;
        property_string_from_parts::<PveQmIde>(data, &device, true)?;
    } else if device.starts_with("sata") {
        property_string_add_missing_data::<QemuConfigSata>(data, &state.record, &form_ctx)?;
        property_string_from_parts::<QemuConfigSata>(data, &device, true)?;
    } else if device.starts_with("scsi") {
        property_string_add_missing_data::<QemuConfigScsi>(data, &state.record, &form_ctx)?;
        property_string_from_parts::<QemuConfigScsi>(data, &device, true)?;
    } else if device.starts_with("virtio") {
        property_string_add_missing_data::<QemuConfigVirtio>(data, &state.record, &form_ctx)?;
        property_string_from_parts::<QemuConfigVirtio>(data, &device, true)?;
    } else {
        bail!("assemble_device_data: unsupported device type '{device}'");
    }
    let data = delete_empty_values(data, &[&device], false);
    Ok(data)
}

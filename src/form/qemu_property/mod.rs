use std::rc::Rc;

use anyhow::bail;
use proxmox_schema::{ApiType, ObjectSchema, Schema};
use regex::Regex;
use serde_json::Value;

use pwt::prelude::*;
use pwt::widget::form::{delete_empty_values, Field, FormContext, Number};
use pwt::widget::Column;

use proxmox_yew_comp::SchemaValidation;

use pve_api_types::{QemuConfig, StorageContent};

use crate::form::{
    format_hotplug_feature, format_qemu_ostype, property_string_load_hook,
    property_string_submit_hook, BootDeviceList, HotplugFeatureSelector, PveStorageSelector,
    QemuOstypeSelector,
};
use crate::widgets::{EditableProperty, RenderPropertyInputPanelFn};
use crate::QemuConfigStartup;

mod qemu_smbios1_property;
pub use qemu_smbios1_property::qemu_smbios_property;

mod qemu_spice_enhancement_property;
pub use qemu_spice_enhancement_property::qemu_spice_enhancement_property;

mod qemu_amd_sev_property;
pub use qemu_amd_sev_property::qemu_amd_sev_property;

mod qemu_memory_property;
pub use qemu_memory_property::qemu_memory_property;

mod qemu_agent_property;
pub use qemu_agent_property::qemu_agent_property;

mod qemu_bios_property;
pub use qemu_bios_property::qemu_bios_property;

fn lookup_schema(name: &str) -> Option<(bool, &'static Schema)> {
    let allof_schema = QemuConfig::API_SCHEMA.unwrap_all_of_schema();

    for entry in allof_schema.list {
        if let Schema::Object(object_schema) = entry {
            if let Some((optional, schema)) = lookup_object_property_schema(&object_schema, name) {
                return Some((optional, schema));
            }
        }
    }
    None
}

fn lookup_object_property_schema(
    object_schema: &ObjectSchema,
    name: &str,
) -> Option<(bool, &'static Schema)> {
    if let Ok(ind) = object_schema
        .properties
        .binary_search_by_key(&name, |(n, _, _)| n)
    {
        let (_name, optional, schema) = object_schema.properties[ind];
        return Some((optional, schema));
    }
    None
}

fn render_string_input_panel(name: &'static str) -> RenderPropertyInputPanelFn {
    RenderPropertyInputPanelFn::new(move |_, _| {
        let mut input = Field::new().name(name.to_string()).submit_empty(true);

        if let Some((optional, schema)) = lookup_schema(&name) {
            input.set_schema(schema);
            input.set_required(!optional);
        }
        input.into()
    })
}

pub fn qemu_onboot_property() -> EditableProperty {
    EditableProperty::new_bool("onboot", tr!("Start on boot"), false).required(true)
}

pub fn qemu_tablet_property() -> EditableProperty {
    EditableProperty::new_bool("tablet", tr!("Use tablet for pointer"), true).required(true)
}

pub fn qemu_acpi_property() -> EditableProperty {
    EditableProperty::new_bool("acpi", tr!("ACPI support"), true).required(true)
}

pub fn qemu_kvm_property() -> EditableProperty {
    EditableProperty::new_bool("kvm", tr!("KVM hardware virtualization"), true).required(true)
}
pub fn qemu_freeze_property() -> EditableProperty {
    EditableProperty::new_bool("freeze", tr!("Freeze CPU on startup"), false).required(true)
}

pub fn qemu_localtime_property() -> EditableProperty {
    EditableProperty::new_bool("localtime", tr!("Use local time for RTC"), false).required(true)
}

pub fn qemu_protection_property() -> EditableProperty {
    EditableProperty::new_bool("protection", tr!("Protection"), false).required(true)
}

pub fn qemu_name_property(vmid: u32) -> EditableProperty {
    EditableProperty::new("name", tr!("Name"))
        .required(true)
        .placeholder(format!("VM {}", vmid))
        .render_input_panel(render_string_input_panel("name"))
        .submit_hook(|form_ctx: FormContext| {
            let data = form_ctx.get_submit_data();
            Ok(delete_empty_values(&data, &["name"], false))
        })
}

pub fn qemu_ostype_property() -> EditableProperty {
    EditableProperty::new("ostype", tr!("OS Type"))
        .required(true)
        .placeholder("Other")
        .renderer(|_, v, _| match v.as_str() {
            Some(s) => format_qemu_ostype(s).into(),
            None => v.into(),
        })
        .render_input_panel(move |_, _| {
            QemuOstypeSelector::new()
                .style("width", "100%")
                .name("ostype")
                .submit_empty(true)
                .into()
        })
        .submit_hook(|form_ctx: FormContext| {
            let data = form_ctx.get_submit_data();
            Ok(delete_empty_values(&data, &["ostype"], false))
        })
}

pub fn qemu_startup_property() -> EditableProperty {
    EditableProperty::new("startup", tr!("Start/Shutdown order"))
        .required(true)
        .placeholder("order=any")
        .render_input_panel(|_, _| {
            Column::new()
                .gap(2)
                .class(pwt::css::Flex::Fill)
                .class(pwt::css::AlignItems::Stretch)
                .with_child(crate::widgets::label_field(
                    tr!("Order"),
                    Number::<u32>::new()
                        .name("_startup_order")
                        .placeholder(tr!("any")),
                ))
                .with_child(crate::widgets::label_field(
                    tr!("Startup delay"),
                    Number::<u32>::new()
                        .name("_startup_up")
                        .placeholder(tr!("default")),
                ))
                .with_child(crate::widgets::label_field(
                    tr!("Shutdown timeout"),
                    Number::<u32>::new()
                        .name("_startup_down")
                        .placeholder(tr!("default")),
                ))
                .into()
        })
        .load_hook(property_string_load_hook::<QemuConfigStartup>("startup"))
        .submit_hook(property_string_submit_hook::<QemuConfigStartup>(
            "startup", true,
        ))
}

pub fn qemu_boot_property() -> EditableProperty {
    EditableProperty::new("boot", tr!("Boot Order"))
        .placeholder(format!(
            "{}, {}, {}",
            tr!("first Disk"),
            tr!("any CD-ROM"),
            tr!("any net")
        ))
        .render_input_panel(move |_, record: Rc<Value>| {
            BootDeviceList::new(record.clone())
                .name("boot")
                .submit_empty(true)
                .into()
        })
        .required(true)
        .submit_hook(|form_ctx: FormContext| {
            let data = form_ctx.get_submit_data();
            Ok(delete_empty_values(&data, &["boot"], false))
        })
}

pub fn qemu_hotplug_property() -> EditableProperty {
    EditableProperty::new("hotplug", tr!("Hotplug"))
        .placeholder(format_hotplug_feature(&Value::Null))
        .renderer(|_, v, _| format_hotplug_feature(v).into())
        .load_hook(|mut record: Value| {
            record["hotplug"] = crate::form::normalize_hotplug_value(&record["hotplug"]);
            Ok(record)
        })
        .render_input_panel(move |_, _| {
            HotplugFeatureSelector::new()
                .name("hotplug")
                .submit_empty(true)
                .into()
        })
        .required(true)
        .submit_hook(|form_ctx: FormContext| {
            let data = form_ctx.get_submit_data();
            Ok(delete_empty_values(&data, &["hotplug"], false))
        })
}

pub fn qemu_startdate_property() -> EditableProperty {
    thread_local! {
        static QEMU_STARTDATE_MATCH: Regex = Regex::new(r#"^(now|\d{4}-\d{1,2}-\d{1,2}(T\d{1,2}:\d{1,2}:\d{1,2})?)$"#).unwrap();
    }
    EditableProperty::new("startdate", tr!("RTC start date"))
        .placeholder("now")
        // Note current schema definition does not include the regex, so we
        // need to add a validate function to the field.
        .render_input_panel(move |_, _| {
            Field::new()
                .name("startdate")
                .placeholder("now")
                .submit_empty(true)
                .validate(|v: &String| {
                    if QEMU_STARTDATE_MATCH.with(|r| r.is_match(v)) {
                        return Ok(());
                    }
                    bail!(tr!("Format") + ": \"now\" or \"2006-06-17T16:01:21\" or \"2006-06-17\"")
                })
                .into()
        })
        .required(true)
        .submit_hook(|form_ctx: FormContext| {
            let data = form_ctx.get_submit_data();
            Ok(delete_empty_values(&data, &["startdate"], false))
        })
}

pub fn qemu_vmstatestorage_property(node: &str) -> EditableProperty {
    EditableProperty::new("vmstatestorage", tr!("VM State storage"))
        .required(true)
        .placeholder(tr!("Automatic"))
        .render_input_panel({
            let node = node.to_owned();
            move |_, _| {
                PveStorageSelector::new(node.clone())
                    .mobile(true)
                    .name("vmstatestorage")
                    .submit_empty(true)
                    .content_types(vec![StorageContent::Images])
                    .placeholder(tr!("Automatic (Storage used by the VM, or 'local')"))
                    .into()
            }
        })
        .submit_hook(|form_ctx: FormContext| {
            let data = form_ctx.get_submit_data();
            Ok(delete_empty_values(&data, &["vmstatestorage"], false))
        })
}

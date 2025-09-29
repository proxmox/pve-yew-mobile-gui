mod boot_device_list;
use anyhow::{bail, Error};
pub use boot_device_list::{BootDeviceList, PveBootDeviceList};

mod qemu_ostype_selector;
pub use qemu_ostype_selector::{format_qemu_ostype, QemuOstypeSelector};

mod qemu_cpu_flags_list;
pub use qemu_cpu_flags_list::QemuCpuFlags;

mod qemu_cpu_model_selector;
pub use qemu_cpu_model_selector::QemuCpuModelSelector;

mod qemu_display_type_selector;
pub use qemu_display_type_selector::{format_qemu_display_type, QemuDisplayTypeSelector};

mod qemu_machine_version_selector;
pub use qemu_machine_version_selector::QemuMachineVersionSelector;

mod pve_network_selector;
pub use pve_network_selector::PveNetworkSelector;

mod pve_vlan_field;
pub use pve_vlan_field::PveVlanField;

mod hotplug_feature_selector;
pub use hotplug_feature_selector::{
    format_hotplug_feature, normalize_hotplug_value, HotplugFeatureSelector,
    PveHotplugFeatureSelector,
};

mod qemu_property;
pub use qemu_property::{
    qemu_acpi_property, qemu_agent_property, qemu_amd_sev_property, qemu_bios_property,
    qemu_boot_property, qemu_cpu_flags_property, qemu_display_property, qemu_freeze_property,
    qemu_hotplug_property, qemu_kernel_scheduler_property, qemu_kvm_property,
    qemu_localtime_property, qemu_machine_property, qemu_memory_property, qemu_name_property,
    qemu_network_mtu_property, qemu_network_property, qemu_onboot_property, qemu_ostype_property,
    qemu_protection_property, qemu_scsihw_property, qemu_smbios_property,
    qemu_sockets_cores_property, qemu_spice_enhancement_property, qemu_startdate_property,
    qemu_startup_property, qemu_tablet_property, qemu_vmstate_property,
    qemu_vmstatestorage_property,
};

mod pve_storage_selector;
pub use pve_storage_selector::PveStorageSelector;

use proxmox_schema::{property_string::PropertyString, ApiType, Schema};
use pwt::widget::form::{delete_empty_values, FormContext};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

use proxmox_client::ApiResponseData;
use proxmox_schema::ObjectSchemaType;
use proxmox_yew_comp::ApiLoadCallback;

use yew::Callback;

use crate::widgets::PropertyEditorState;

// fixme: move to proxmox-yew-comp::form
pub fn typed_load<T: DeserializeOwned + Serialize>(
    url: impl Into<String>,
) -> ApiLoadCallback<Value> {
    let url = url.into();
    let url_cloned = url.clone();
    ApiLoadCallback::new(move || {
        let url = url.clone();
        async move {
            // use Rust type to correctly convert pve boolean 0, 1 values
            let resp: ApiResponseData<T> = proxmox_yew_comp::http_get_full(url, None).await?;

            Ok(ApiResponseData {
                data: serde_json::to_value(resp.data)?,
                attribs: resp.attribs,
            })
        }
    })
    .url(url_cloned)
}

pub fn property_string_load_hook<P: ApiType + Serialize + DeserializeOwned>(
    name: impl Into<String>,
) -> Callback<Value, Result<Value, Error>> {
    let name = name.into();
    Callback::from(move |mut record: Value| {
        flatten_property_string::<P>(&mut record, &name)?;
        Ok(record)
    })
}

pub fn property_string_submit_hook<P: ApiType + Serialize + DeserializeOwned>(
    name: impl Into<String>,
    delete_empty: bool,
) -> Callback<PropertyEditorState, Result<Value, Error>> {
    let name = name.into();
    Callback::from(move |state: PropertyEditorState| {
        let mut data = state.get_submit_data();
        // fixme: add defaults
        property_string_from_parts::<P>(&mut data, &name, true)?;
        if delete_empty {
            data = delete_empty_values(&data, &[&name], false);
        }
        Ok(data)
    })
}

// Copied from proxmox-yew-com, added proper error handling

/// Convert a property string to separate properties
///
/// This is useful for use in an [`crate::widgets::EditDialog`] when editing parts of a property string.
/// Takes the `name` property from `data`, parses it as property string, and sets it back to
/// `data` as `_{key}`, so that this can be used as a field. If it's not desired
/// to expose a property to the UI, simply add a hidden field to the form, or use
/// [property_string_add_missing_data] to re-add missing data before submit.
pub fn flatten_property_string<T: ApiType + Serialize + DeserializeOwned>(
    data: &mut Value,
    name: &str,
) -> Result<(), Error> {
    // Note: schema.parse_property_string does not work for schemas using KeyAliasInfo!!
    let test: Option<PropertyString<T>> = serde_json::from_value(data[name].clone())?;
    if let Some(test) = test {
        let record: Value = serde_json::to_value(test.into_inner())?;
        match record {
            Value::Object(map) => {
                for (part, v) in map {
                    data[format!("_{part}")] = v;
                }
            }
            _ => bail!("flatten_property_string {name:?} failed: result is not an Object"),
        }
    }

    Ok(())
}

/// Copy undefined object values from other object.
///
/// This is useful to assemble data inside form submit functions. Instead of adding Hidden fields to
/// include data, this function copies undefined values from the provided object (which
/// usually refers to the initial data loaded by the form).
pub fn property_string_add_missing_data<T: ApiType + Serialize + DeserializeOwned>(
    submit_data: &mut Value,
    original_data: &Value,
    form_ctx: &FormContext,
) -> Result<(), Error> {
    let props = match T::API_SCHEMA {
        Schema::Object(object_schema) => object_schema.properties(),
        _ => {
            bail!("property_string_add_missing_data: internal error - got unsupported schema type")
        }
    };

    let form_ctx = form_ctx.read();
    if let Value::Object(map) = submit_data {
        if let Value::Object(original_map) = original_data {
            for (part, _, _) in props {
                let part = format!("_{part}");
                if !map.contains_key(&part)
                    && !form_ctx.contains_field(&part)
                    && original_map.contains_key(&part)
                {
                    map.insert(part.clone(), original_data[&part].clone());
                }
            }
        }
    }
    Ok(())
}

// Copied from proxmox-yew-com and added proper error handling
/// Uses an [`proxmox_schema::ObjectSchema`] to generate a property string from separate properties.
///
/// This is useful for use in an [`crate::widgets::EditDialog`] when editing parts of a property string.
/// Takes the single properties from `data` and assembles a property string.
///
/// Property string data is removed from the original data, and re-added as assembled
/// property string with name `name`.
///
/// Returns the parsed rust type.
///
/// Uses "_{key}"" for property names like [flatten_property_string].
pub fn property_string_from_parts<T: ApiType + Serialize + DeserializeOwned>(
    data: &mut Value,
    name: &str,
    skip_empty_values: bool,
) -> Result<Option<T>, Error> {
    let props = match T::API_SCHEMA {
        Schema::Object(object_schema) => object_schema.properties(),
        _ => bail!("property_string_from_parts: internal error - got unsupported schema type"),
    };

    if let Value::Object(map) = data {
        let mut value = json!({});

        let mut has_parts = false;
        for (part, _, _) in props {
            if let Some(v) = map.remove(&format!("_{part}")) {
                has_parts = true;
                let is_empty = match &v {
                    Value::Null => true,
                    Value::String(s) => s.is_empty(),
                    _ => false,
                };
                if !(skip_empty_values && is_empty) {
                    value[part] = v;
                }
            }
        }

        if !has_parts {
            data[name] = "".into();
            return Ok(None);
        }

        let option: Option<T> = serde_json::from_value(value)?;
        data[name] = match option {
            Some(ref parsed) => proxmox_schema::property_string::print::<T>(parsed)?,
            None::<_> => String::new(),
        }
        .into();

        Ok(option)
    } else {
        bail!("property_string_from_parts: data is no Object");
    }
}

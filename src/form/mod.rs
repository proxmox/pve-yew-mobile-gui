mod boot_device_list;
use anyhow::{bail, Error};
pub use boot_device_list::{BootDeviceList, PveBootDeviceList};

mod qemu_ostype_selector;
pub use qemu_ostype_selector::{format_qemu_ostype, QemuOstypeSelector};

mod hotplug_feature_selector;
pub use hotplug_feature_selector::{
    format_hotplug_feature, normalize_hotplug_value, HotplugFeatureSelector,
    PveHotplugFeatureSelector,
};

mod qemu_smbios1_edit;
pub use qemu_smbios1_edit::qemu_smbios_property;

mod qemu_spice_enhancement_edit;
pub use qemu_spice_enhancement_edit::qemu_spice_enhancement_property;

mod qemu_amd_sev_edit;
pub use qemu_amd_sev_edit::qemu_amd_sev_property;

mod qemu_memory_edit;
pub use qemu_memory_edit::qemu_memory_property;

mod pve_storage_selector;
pub use pve_storage_selector::PveStorageSelector;

use proxmox_schema::{ApiType, Schema};
use pwt::widget::form::{delete_empty_values, FormContext};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

use proxmox_client::ApiResponseData;
use proxmox_schema::ObjectSchemaType;
use proxmox_yew_comp::ApiLoadCallback;

use yew::Callback;

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
        flatten_property_string(&mut record, &name, &P::API_SCHEMA)?;
        Ok(record)
    })
}

pub fn property_string_submit_hook<P: ApiType + Serialize + DeserializeOwned>(
    name: impl Into<String>,
    delete_empty: bool,
) -> Callback<FormContext, Result<Value, Error>> {
    let name = name.into();
    Callback::from(move |form_ctx: FormContext| {
        let mut data = form_ctx.get_submit_data();
        property_string_from_parts::<P>(&mut data, &name, true)?;
        if delete_empty {
            data = delete_empty_values(&data, &[&name], false);
        }
        Ok(data)
    })
}

/// Property String Property Name
///
/// Create name to store parts of a property (`_{prop_name}_{part}`).
pub fn pspn(prop_name: &str, part: &str) -> String {
    format!("_{prop_name}_{part}")
}

// Copied from proxmox-yew-com, added proper error handling

/// Convert a property string to separate properties
///
/// This is useful for use in an [`crate::widgets::EditDialog`] when editing parts of a property string.
/// Takes the `name` property from `data`, parses it as property string, and sets it back to
/// `data` as `_{name}_{key}` (see [pspn]), so this can be used as a field. If it's not desired
/// to expose a property to the UI, simply add a hidden field to the form.
pub fn flatten_property_string(
    data: &mut Value,
    name: &str,
    schema: &'static Schema,
) -> Result<(), Error> {
    if let Some(prop_str) = data[name].as_str() {
        match schema.parse_property_string(prop_str)? {
            Value::Object(map) => {
                for (part, v) in map {
                    data[pspn(name, &part)] = v;
                }
            }
            _other => {
                bail!("flatten_property_string {name:?} failed: result is not an Object");
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
/// Uses [pspn] for property names like [flatten_property_string].
pub fn property_string_from_parts<T: ApiType + Serialize + DeserializeOwned>(
    data: &mut Value,
    name: &str,
    skip_empty_values: bool,
) -> Result<(), Error> {
    let props = match T::API_SCHEMA {
        Schema::Object(object_schema) => object_schema.properties(),
        _ => bail!("property_string_from_parts: internal error - got unsupported schema type"),
    };

    if let Value::Object(map) = data {
        let mut value = json!({});

        let mut has_parts = false;
        for (part, _, _) in props {
            if let Some(v) = map.remove(&pspn(name, part)) {
                has_parts = true;
                let is_empty = match &v {
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
            return Ok(());
        }

        let option: Option<T> = serde_json::from_value(value)?;
        data[name] = match option {
            Some(parsed) => proxmox_schema::property_string::print::<T>(&parsed)?,
            None => String::new(),
        }
        .into();

        Ok(())
    } else {
        bail!("property_string_from_parts: data is no Object");
    }
}

mod boot_device_list;
use anyhow::{bail, format_err, Error};
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

mod pve_storage_selector;
pub use pve_storage_selector::PveStorageSelector;

use proxmox_schema::{ApiType, Schema};
use pwt::widget::form::{delete_empty_values, FormContext};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use proxmox_client::ApiResponseData;
use proxmox_yew_comp::{form::property_string_from_parts, ApiLoadCallback};
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
        // fixme: raise errors?
        property_string_from_parts::<P>(&mut data, &name, true);
        if delete_empty {
            let is_empty = match data.get(&name) {
                Some(Value::Null) => true,
                Some(Value::String(s)) => s.is_empty(),
                _ => false,
            };
            if is_empty {
                delete_empty_values(&data, &[&name], false);
            }
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
/// This is useful for use in an [`crate::EditWindow`] when editing parts of a property string.
/// Takes the `name` property from `data`, parses it as property string, and sets it back to
/// `data` as `_{name}_{key}` (see [pnpn]), so this can be used as a field. If it's not desired
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

pub fn parse_property_string_value<T: ApiType + DeserializeOwned>(
    value: &Value,
) -> Result<T, Error> {
    if let Value::String(value_str) = value {
        parse_property_string(value_str)
    } else {
        Err(format_err!(
            "parse_property_string: value is no string type"
        ))
    }
}

pub fn parse_property_string<T: ApiType + DeserializeOwned>(
    value_str: impl AsRef<str>,
) -> Result<T, Error> {
    match T::API_SCHEMA.parse_property_string(value_str.as_ref()) {
        Ok(props) => return serde_json::from_value::<T>(props).map_err(|e| e.into()),
        Err(e) => return Err(e),
    }
}

mod boot_device_list;
use anyhow::{format_err, Error};
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

use proxmox_schema::ApiType;
use pwt::{
    props::SubmitCallback,
    widget::form::{delete_empty_values, FormContext},
};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use proxmox_client::ApiResponseData;
use proxmox_yew_comp::{
    form::{flatten_property_string, property_string_from_parts},
    http_put, ApiLoadCallback,
};
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

pub fn load_property_string<
    T: DeserializeOwned + Serialize,
    P: ApiType + Serialize + DeserializeOwned,
>(
    url: impl Into<String>,
    name: impl Into<String>,
) -> ApiLoadCallback<Value> {
    let url = url.into();
    let url_cloned = url.clone();
    let name = name.into();

    ApiLoadCallback::new(move || {
        let url = url.clone();
        let name = name.clone();
        async move {
            let mut resp = typed_load::<T>(url).apply().await?;
            flatten_property_string(&mut resp.data, &name, &P::API_SCHEMA);
            Ok(resp)
        }
    })
    .url(url_cloned)
}

pub fn submit_property_string_hook<P: ApiType + Serialize + DeserializeOwned>(
    name: impl Into<String>,
    delete_empty: bool,
) -> Callback<FormContext, Result<Value, Error>> {
    let name = name.into();
    Callback::from(move |form_ctx: FormContext| {
        let mut data = form_ctx.get_submit_data();
        property_string_from_parts::<P>(&mut data, "agent", true);
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

pub fn submit_property_string<P: ApiType + Serialize + DeserializeOwned>(
    url: impl Into<String>,
    name: impl Into<String>,
) -> SubmitCallback<FormContext> {
    let url = url.into();
    let name = name.into();
    SubmitCallback::new(move |ctx: FormContext| {
        let url = url.clone();
        let name = name.clone();
        async move {
            let mut value = ctx.get_submit_data();
            property_string_from_parts::<P>(&mut value, &name, true);
            // fixme: do we rellay need/want this here?
            let value = delete_empty_values(&value, &[&name], false);
            http_put(url.clone(), Some(value)).await
        }
    })
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

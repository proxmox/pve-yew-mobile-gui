mod boot_device_list;
pub use boot_device_list::{BootDeviceList, PveBootDeviceList};

mod qemu_config_ostype_selector;
pub use qemu_config_ostype_selector::QemuConfigOstypeSelector;

mod hotplug_feature_selector;
pub use hotplug_feature_selector::{HotplugFeatureSelector, PveHotplugFeatureSelector};

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

// fixme: move to proxmox-yew-comp::form
pub fn typed_load<T: DeserializeOwned + Serialize>(
    url: impl Into<String>,
) -> ApiLoadCallback<Value> {
    let url = url.into();
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
            let value = delete_empty_values(&value, &[&name], false);
            http_put(url.clone(), Some(value)).await
        }
    })
}

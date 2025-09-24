use serde::{Deserialize, Serialize};
use serde_json::Value;

use proxmox_schema::{api, ApiStringFormat};

// fixme: define all those types in pve-api-types

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct StorageEntry {
    pub format: String,
    pub content: String,
    pub size: i64,
    pub volid: String,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ServiceStatus {
    pub state: String,
    pub active_state: String,
    pub unit_state: String,
    pub name: String,
    pub service: String,
    pub desc: String,
}

#[api {
    default_key: "order",
    properties: {
        order: {
            type: u32,
            optional: true,
            minimum: 0,
        },
           up: {
            type: u32,
            optional: true,
            minimum: 0,
        },
        down: {
            type: u32,
            optional: true,
            minimum: 0,
        },
    }
}]
#[derive(Deserialize, Serialize, PartialEq, Clone)]
/// Qemu Startup ordering
pub struct QemuConfigStartup {
    /// Order is a non-negative number defining the general startup order. Shutdown in done with reverse ordering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
    /// Delay (seconds) to wait before the next VM is started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up: Option<u32>,
    /// Delay (seconds) to wait before the next VM is stopped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub down: Option<u32>,
}

#[api]
#[derive(Deserialize, Serialize, PartialEq, Clone)]
/// Qemu CPU model info (GET /api2/json/nodes/{node}/capabilities/qemu/cpu
pub struct QemuCpuModel {
    /// True if this is a custom CPU model.
    #[serde(deserialize_with = "proxmox_serde::perl::deserialize_bool")]
    pub custom: bool,
    /// Name of the CPU model. Identifies it for subsequent API calls. Prefixed with 'custom-' for custom models.
    pub name: String,
    /// CPU vendor visible to the guest when this model is selected. Vendor of 'reported-model' in case of custom models.
    pub vendor: String,
}

#[api]
/// Qemu Machine Type (q35 or pc)
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum QemuMachineType {
    #[serde(rename = "q35")]
    /// Q35
    Q35,
    #[serde(rename = "i440fx")]
    /// I440FX
    I440fx,
    #[serde(rename = "virt")]
    /// Virt (Arm)
    ///
    /// Note: Not returned by the api for now.
    Virt,
}
serde_plain::derive_display_from_serialize!(QemuMachineType);
serde_plain::derive_fromstr_from_deserialize!(QemuMachineType);

#[api(
    properties: {
        type: {
            format: &ApiStringFormat::PropertyString(&QemuMachineType::API_SCHEMA),
            type: String,
        }
    }
)]
#[derive(Deserialize, Serialize, PartialEq, Clone)]
/// Machine type info (GET /api2/json/nodes/{node}/capabilities/qemu/machines
pub struct QemuMachineInfo {
    /// Full name of machine type and version.
    pub id: String,
    /// Machine type
    #[serde(rename = "type")]
    pub ty: QemuMachineType,
    /// The machine version.
    pub version: String,
    /// Notable changes of a version, currently only set for +pveX versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
/// Get the virtual machine configuration with both current and pending values.
///
/// (`GET /api2/json/nodes/{node}/qemu/{vmid}/pending) -> Vec<QemuPendingConfigValue>`
pub struct QemuPendingConfigValue {
    /// Configuration option name.
    pub key: String,
    /// Indicates a pending delete request if present and not 0. The value 2 indicates a force-delete request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete: Option<u8>,
    /// Current value.
    pub value: Option<Value>,
    /// Pending value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<Value>,
}

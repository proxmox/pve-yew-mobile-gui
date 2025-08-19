use serde::{Deserialize, Serialize};

use proxmox_schema::api;

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

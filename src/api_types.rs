use serde::{Deserialize, Serialize};
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

use serde::{Deserialize, Serialize};
// fixme: define all those types in pve-api-types

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct StorageEntry {
    pub format: String,
    pub content: String,
    pub size: i64,
    pub volid: String,
}

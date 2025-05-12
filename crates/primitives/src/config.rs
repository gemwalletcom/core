use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::PlatformStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub releases: Vec<Release>,
    pub versions: ConfigVersions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub version: String,
    pub store: PlatformStore,
    pub upgrade_required: bool,
}

impl Release {
    pub fn new(store: PlatformStore, version: String, upgrade_required: bool) -> Self {
        Self {
            version,
            store,
            upgrade_required,
        }
    }
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigVersions {
    pub fiat_on_ramp_assets: i32,
    pub fiat_off_ramp_assets: i32,
    pub swap_assets: i32,
}

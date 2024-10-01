use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    IOS,
    Android,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum PlatformStore {
    AppStore,
    GooglePlay,
    Fdroid,
    Huawei,
    SolanaStore,
    SamsungStore,
    ApkUniversal,
}

impl PlatformStore {
    pub fn all() -> Vec<PlatformStore> {
        PlatformStore::iter().collect()
    }
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::IOS => "ios",
            Platform::Android => "android",
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            Platform::IOS => 1,
            Platform::Android => 2,
        }
    }

    pub fn new(s: &str) -> Option<Platform> {
        match s {
            "ios" => Some(Platform::IOS),
            "android" => Some(Platform::Android),
            _ => None,
        }
    }
}

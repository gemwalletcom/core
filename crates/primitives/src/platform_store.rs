use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

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
    Emerald,
    Local,
}

impl PlatformStore {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum AssetType {
    NATIVE,
    ERC20,
    BEP20,
    SPL,
    TRC20,
    TOKEN,
    IBC,
    JETTON,
    SYNTH,
}

impl AssetType {
    pub fn all() -> Vec<AssetType> {
        AssetType::iter().collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetSubtype {
    NATIVE,
    TOKEN,
}

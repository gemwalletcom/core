use crate::AssetType;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SolanaTokenProgramId {
    Token,
    Token2022,
}

impl SolanaTokenProgramId {
    pub fn from_asset_type(asset_type: &AssetType) -> Option<Self> {
        match asset_type {
            AssetType::SPL => Some(Self::Token),
            AssetType::SPL2022 => Some(Self::Token2022),
            _ => None,
        }
    }
}

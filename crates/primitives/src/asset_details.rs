use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Asset, AssetId, AssetMarket, AssetScore, LinkType, Price};

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetFull {
    pub asset: Asset,
    pub properties: AssetProperties,
    pub score: AssetScore,
    pub tags: Vec<String>,
    pub links: Vec<AssetLink>,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetBasic {
    pub asset: Asset,
    pub properties: AssetProperties,
    pub score: AssetScore,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMarketPrice {
    pub price: Option<Price>,
    pub market: Option<AssetMarket>,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetProperties {
    pub is_enabled: bool,
    pub is_buyable: bool,
    pub is_sellable: bool,
    pub is_swapable: bool,
    pub is_stakeable: bool,

    pub staking_apr: Option<f64>,
}

impl AssetProperties {
    pub fn default(asset_id: AssetId) -> Self {
        let is_stakeable = asset_id.is_native() && asset_id.chain.is_stake_supported();
        Self {
            is_enabled: true,
            is_buyable: false,
            is_sellable: false,
            is_swapable: asset_id.chain.is_swap_supported(),
            is_stakeable,
            staking_apr: None,
        }
    }
}

#[typeshare(swift = "Sendable, Equatable, Hashable")]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetLink {
    pub name: String,
    pub url: String,
}

impl AssetLink {
    pub fn new(url: &str, link_type: LinkType) -> Self {
        Self {
            name: link_type.name(),
            url: url.to_string(),
        }
    }
}

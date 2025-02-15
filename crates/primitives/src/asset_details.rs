use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Asset, AssetMarket, AssetScore, Chain, Price};

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetFull {
    pub asset: Asset,
    pub links: Vec<AssetLink>,
    pub properties: AssetProperties,
    pub score: AssetScore,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetBasic {
    pub asset: Asset,
    pub properties: AssetProperties,
    pub score: AssetScore,
    #[typeshare(skip)]
    pub links: Vec<AssetLink>,
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
    pub fn default(chain: Chain) -> Self {
        Self {
            is_enabled: true,
            is_buyable: false,
            is_sellable: false,
            is_swapable: chain.is_swap_supported(),
            is_stakeable: chain.is_stake_supported(),
            staking_apr: None,
        }
    }
}

pub const ASSET_LINK_X: &str = "x";
pub const ASSET_LINK_FACEBOOK: &str = "facebook";
pub const ASSET_LINK_WEBSITE: &str = "website";
pub const ASSET_LINK_EXPLORER: &str = "explorer";
pub const ASSET_LINK_TELEGRAM: &str = "telegram";
pub const ASSET_LINK_GITHUB: &str = "github";
pub const ASSET_LINK_YOUTUBE: &str = "youtube";
pub const ASSET_LINK_REDDIT: &str = "reddit";
pub const ASSET_LINK_COINGECKO: &str = "coingecko";
pub const ASSET_LINK_COINMARKETCAP: &str = "coinmarketcap";
pub const ASSET_LINK_DISCORD: &str = "discord";

#[typeshare(swift = "Sendable, Equatable, Hashable")]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetLink {
    pub name: String,
    pub url: String,
}

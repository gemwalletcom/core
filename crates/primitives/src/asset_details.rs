use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Asset, AssetMarket, AssetScore, Price};

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetFull {
    pub asset: Asset,
    //TODO: Remove from this in the future
    #[typeshare(skip)]
    pub details: Option<AssetDetails>,
    pub links: Vec<AssetLink>,
    pub properties: AssetProperties,
    pub score: AssetScore,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetBasic {
    pub asset: Asset,
    //TODO: Remove from this in the future
    #[typeshare(skip)]
    pub details: Option<AssetDetails>,
    pub properties: AssetProperties,
    pub score: AssetScore,
}

#[typeshare(skip)]
#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssetDetails {
    pub links: AssetLinks,

    pub is_buyable: bool,
    pub is_sellable: bool,
    pub is_swapable: bool,
    pub is_stakeable: bool,

    pub staking_apr: Option<f64>,
}

#[typeshare(skip)]
#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetLinks {
    pub homepage: Option<String>,
    pub explorer: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub github: Option<String>,
    pub youtube: Option<String>,
    pub facebook: Option<String>,
    pub reddit: Option<String>,
    pub coingecko: Option<String>,
    pub coinmarketcap: Option<String>,
    pub discord: Option<String>,
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
    pub is_buyable: bool,
    pub is_sellable: bool,
    pub is_swapable: bool,
    pub is_stakeable: bool,

    pub staking_apr: Option<f64>,
}

pub const ASSET_LINK_TWITTER: &str = "twitter";
pub const ASSET_LINK_FACEBOOK: &str = "facebook";
pub const ASSET_LINK_HOMEPAGE: &str = "homepage";
pub const ASSET_LINK_EXPLORER: &str = "explorer";
pub const ASSET_LINK_TELEGRAM: &str = "telegram";
pub const ASSET_LINK_GITHUB: &str = "github";
pub const ASSET_LINK_YOUTUBE: &str = "youtube";
pub const ASSET_LINK_REDDIT: &str = "reddit";
pub const ASSET_LINK_COINGECKO: &str = "coingecko";
pub const ASSET_LINK_COINMARKETCAP: &str = "coinmarketcap";
pub const ASSET_LINK_DISCORD: &str = "discord";

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetLink {
    pub name: String,
    pub url: String,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDetailsInfo {
    pub details: AssetDetails,
    pub market: AssetMarket,
}

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Asset, AssetMarket, AssetScore, Price};

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetFull {
    pub asset: Asset,
    pub details: Option<AssetDetails>,
    pub price: Option<Price>,
    pub market: Option<AssetMarket>,
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
pub struct AssetDetails {
    pub links: AssetLinks,

    pub is_buyable: bool,
    pub is_sellable: bool,
    pub is_swapable: bool,
    pub is_stakeable: bool,

    pub staking_apr: Option<f64>,
}

impl AssetDetails {
    pub fn from_links(links: AssetLinks) -> Self {
        AssetDetails {
            links,
            is_buyable: false,
            is_sellable: false,
            is_swapable: false,
            is_stakeable: false,
            staking_apr: None,
        }
    }
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct AssetDetailsInfo {
    pub details: AssetDetails,
    pub market: AssetMarket,
}

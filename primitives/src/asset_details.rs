use typeshare::typeshare;
use serde::{Serialize, Deserialize};

use crate::{Asset, AssetMarket, Price};

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFull {  
    pub asset: Asset,
    pub details: Option<AssetDetails>,
    pub price: Option<Price>,
    pub market: Option<AssetMarket>
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDetails {
    pub links: AssetLinks,
}

#[typeshare]
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

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDetailsInfo {
    pub details: AssetDetails,
    pub market: AssetMarket,
}
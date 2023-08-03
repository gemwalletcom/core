use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct AssetPrice {
    pub asset_id: String,
    pub price: f64,
    pub price_change_percentage_24h: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct AssetPrices {
    pub currency: String,
    pub prices: Vec<AssetPrice>,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable")]
#[serde(rename_all = "camelCase")]
pub struct AssetPricesRequest {
    pub currency: String,
    pub asset_ids: Vec<String>,
}


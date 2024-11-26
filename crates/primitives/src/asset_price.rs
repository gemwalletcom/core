use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::Price;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AssetPrice {
    pub asset_id: String,
    pub price: f64,
    pub price_change_percentage_24h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AssetMarket {
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<i32>,
    pub total_volume: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AssetPrices {
    pub currency: String,
    pub prices: Vec<AssetPrice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AssetPricesRequest {
    pub currency: Option<String>,
    pub asset_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Charts {
    pub price: Option<Price>,
    pub market: Option<AssetMarket>,
    pub prices: Vec<ChartValue>,
    pub market_caps: Vec<ChartValue>,
    pub total_volumes: Vec<ChartValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ChartValue {
    pub timestamp: i32,
    pub value: f32,
}

impl PartialEq for ChartValue {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp && self.value == other.value
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "lowercase")]
pub enum ChartPeriod {
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
    All,
}

impl ChartPeriod {
    pub fn new(period: String) -> Option<Self> {
        match period.to_lowercase().as_str() {
            "hour" => Some(Self::Hour),
            "day" => Some(Self::Day),
            "week" => Some(Self::Week),
            "month" => Some(Self::Month),
            "quarter" => Some(Self::Quarter),
            "year" => Some(Self::Year),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    pub fn minutes(&self) -> i32 {
        match self {
            ChartPeriod::Hour => 60,
            ChartPeriod::Day => 1440,
            ChartPeriod::Week => 10_080,
            ChartPeriod::Month => 43_200,
            ChartPeriod::Quarter => 131_400,
            ChartPeriod::Year => 525_600,
            ChartPeriod::All => 10_525_600,
        }
    }
}

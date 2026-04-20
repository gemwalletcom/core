use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifiedToken {
    pub id: String,
    #[serde(default)]
    pub organic_score: f64,
    #[serde(default)]
    pub usd_price: f64,
    pub mcap: Option<f64>,
    pub fdv: Option<f64>,
    pub circ_supply: Option<f64>,
    pub total_supply: Option<f64>,
    #[serde(default)]
    pub stats24h: TokenStats,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    #[serde(default)]
    pub price_change: f64,
}

pub type VerifiedTokensResponse = Vec<VerifiedToken>;

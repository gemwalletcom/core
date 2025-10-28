use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotRouteData {
    pub market_index: u32,
    pub side: SpotSide,
    pub size: String,
    pub price: String,
    pub quote_amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpotSide {
    Buy,
    Sell,
}

impl SpotSide {
    pub fn is_buy(&self) -> bool {
        matches!(self, SpotSide::Buy)
    }
}

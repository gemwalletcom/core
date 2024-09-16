use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlert {
    pub asset_id: String,
    pub price: Option<f64>,
}

pub type PriceAlerts = Vec<PriceAlert>;

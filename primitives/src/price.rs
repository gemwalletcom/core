use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
pub struct Price {
    pub price: f64,
    #[serde(rename = "priceChangePercentage24h")]
    pub price_change_percentage_24h: f64,
}
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiatRate {
    pub symbol: String,
    pub rate: f64,
}

impl FiatRate {
    pub fn multiplier(&self, base: f64) -> f64 {
        self.rate * base
    }
}

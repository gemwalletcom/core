use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiatRate {
    pub symbol: String,
    pub name: String,
    pub rate: f64,
}

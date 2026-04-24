use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PricesResponse {
    pub coins: HashMap<String, CoinPrice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CoinPrice {
    pub price: f64,
}

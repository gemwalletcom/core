use serde::{Deserialize, Serialize};

use super::super::token::SpotToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotMarket {
    pub tokens: Vec<i32>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotMeta {
    pub tokens: Vec<SpotToken>,
    pub universe: Vec<SpotMarket>,
}

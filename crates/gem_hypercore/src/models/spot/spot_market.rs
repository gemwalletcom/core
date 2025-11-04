use serde::{Deserialize, Serialize};

use super::super::token::SpotToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotMarket {
    pub tokens: Vec<i32>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotMetaRaw {
    pub tokens: Vec<SpotToken>,
    pub universe: Vec<SpotMarket>,
}

#[derive(Debug, Clone)]
pub struct SpotMeta {
    tokens: Vec<SpotToken>,
    universe: Vec<SpotMarket>,
}

impl From<SpotMetaRaw> for SpotMeta {
    fn from(value: SpotMetaRaw) -> Self {
        Self {
            tokens: value.tokens,
            universe: value.universe,
        }
    }
}

impl SpotMeta {
    pub fn tokens(&self) -> &[SpotToken] {
        &self.tokens
    }

    pub fn universe(&self) -> &[SpotMarket] {
        &self.universe
    }
}

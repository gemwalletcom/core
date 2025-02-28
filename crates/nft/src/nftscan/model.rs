use primitives::Chain;
use serde::{Deserialize, Serialize};

pub fn get_chain(chain: Chain) -> Option<String> {
    match chain {
        Chain::Ethereum => Some("eth".to_owned()),
        Chain::Solana => Some("sol".to_owned()),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTAttribute {
    pub attribute_name: String,
    pub attribute_value: String,
}

impl NFTAttribute {
    pub fn as_primitive(&self) -> Option<primitives::NFTAttribute> {
        if self.attribute_value.is_empty() || self.attribute_value.len() > 18 {
            return None;
        }
        Some(primitives::NFTAttribute {
            name: self.attribute_name.clone(),
            value: self.attribute_value.clone(),
            percentage: None,
        })
    }
}

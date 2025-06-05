use serde::{Deserialize, Serialize};

use crate::Chain;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChainAddress {
    pub chain: Chain,
    pub address: String,
}

impl ChainAddress {
    pub fn new(chain: Chain, address: String) -> Self {
        Self { chain, address }
    }
}

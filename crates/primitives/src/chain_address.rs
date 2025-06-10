use std::fmt;

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

impl fmt::Display for ChainAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.chain, self.address)
    }
}

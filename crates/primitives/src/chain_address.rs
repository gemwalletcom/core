use serde::{Deserialize, Serialize};

use crate::Chain;

#[derive(Serialize, Deserialize)]
pub struct ChainAddress {
    pub chain: Chain,
    pub address: String,
}

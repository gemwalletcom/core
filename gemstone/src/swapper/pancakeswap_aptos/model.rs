use serde::{Deserialize, Serialize};

pub const PANCAKE_SWAP_APTOS_ADDRESS: &str = "0xc7efb4076dbe143cbcd98cfaaa929ecfc8f299203dfff63b95ccb6bfe19850fa";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPairReserve {
    pub reserve_x: String,
    pub reserve_y: String,
}

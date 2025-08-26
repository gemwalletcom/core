use super::transaction::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeaders {
    #[serde(rename = "current-round")]
    pub current_round: u64,
    pub blocks: Vec<BlockHeader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    #[serde(rename = "genesis-id")]
    pub genesis_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub transactions: Vec<Transaction>,
}

use serde::{Deserialize, Serialize};
use super::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeaders {
    #[serde(rename = "current-round")]
    pub current_round: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub block: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTransactionIds {
    #[serde(rename = "blockTxids")]
    pub block_txids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub transactions: Vec<Transaction>,
}
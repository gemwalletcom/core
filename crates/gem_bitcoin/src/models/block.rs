use serde::{Deserialize, Serialize};

use crate::models::Transaction;

type Int = u64;

// Domain models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinBlock {
    pub previous_block_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinNodeInfo {
    pub blockbook: BitcoinBlockbook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinBlockbook {
    pub in_sync: bool,
    pub last_block_time: String,
    pub best_height: Int,
}

// RPC models
#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    pub blockbook: Blockbook,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Blockbook {
    #[serde(rename = "bestHeight")]
    pub best_height: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub page: u64,
    pub total_pages: u64,
    pub txs: Vec<Transaction>,
}

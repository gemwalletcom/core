use serde::{Deserialize, Serialize};

type Int = u64;

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

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

type Int = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct BitcoinBlock {
    #[serde(rename = "previousBlockHash")]
    pub previous_block_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct BitcoinNodeInfo {
    pub blockbook: BitcoinBlockbook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct BitcoinBlockbook {
    #[serde(rename = "inSync")]
    pub in_sync: bool,
    #[serde(rename = "lastBlockTime")]
    pub last_block_time: String,
    #[serde(rename = "bestHeight")]
    pub best_height: Int,
}

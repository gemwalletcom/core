use serde::{Deserialize, Serialize};
use typeshare::typeshare;

type UInt64 = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronBlock {
    #[serde(rename = "block_header")]
    pub block_header: TronHeaderRawData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronHeaderRawData {
    pub raw_data: TronHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronHeader {
    pub number: UInt64,
    pub version: UInt64,
    #[serde(rename = "txTrieRoot")]
    pub tx_trie_root: String,
    pub witness_address: String,
    #[serde(rename = "parentHash")]
    pub parent_hash: String,
    pub timestamp: UInt64,
}
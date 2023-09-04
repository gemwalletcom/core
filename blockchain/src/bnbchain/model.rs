use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub txs: Vec<Transaction>
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: String,
    pub r#type: String,
    pub fee: i64,
    pub memo: String,
    pub asset: Option<String>,
    pub amount: Option<i64>,
    pub from_addr: String,
    pub to_addr: Option<String>,
    pub block_height: i64,
    pub sequence: i64,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub sync_info: SyncInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncInfo {
    pub latest_block_height: i32,
}
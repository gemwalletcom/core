use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

use super::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub chain_id: i32,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub ledger_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub block_height: String,
    pub transactions: Vec<Transaction>,
}

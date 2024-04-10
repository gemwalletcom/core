use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const TRANSFER_ACTION: &str = "Transfer";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub chunks: Vec<ChunkHeader>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    pub height: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkHeader {
    pub shard_id: i64,
    pub gas_used: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub hash: String,
    pub signer_id: String,
    pub receiver_id: String,
    pub nonce: i64,
    pub actions: Vec<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionDeposit {
    pub deposit: String,
}

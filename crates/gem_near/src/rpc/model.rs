use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub chunks: Vec<ChunkHeader>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    pub height: i64,
    pub timestamp: u64,
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
    pub actions: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Action {
    CreateAccount,
    Transfer {
        deposit: String,
    },
    #[serde(untagged)]
    Other(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessKey {
    pub nonce: i64,
    pub permission: String,
}

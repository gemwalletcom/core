use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSONResult<T> {
    pub result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chainhead {
    pub last: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub seqno: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shards {
    pub shards: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortTransactions {
    pub transactions: Vec<ShortTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortTransaction {
    pub lt: String,
    pub account: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub fee: String,
    pub transaction_id: TransactionId,
    pub out_msgs: Vec<OutMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionId {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutMessage {
    pub source: String,
    pub destination: Option<String>,
    pub value: String,
}

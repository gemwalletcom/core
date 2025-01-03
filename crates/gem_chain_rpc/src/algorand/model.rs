use core::str;

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

pub const TRANSACTION_TYPE_PAY: &str = "pay";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsParams {
    #[serde(rename = "last-round")]
    pub last_round: i64,
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
    pub ts: i64,
    pub rnd: i64,
    pub txns: Option<Vec<TransactionPayload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPayload {
    pub txn: Transaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub amt: Option<i64>,
    pub fee: Option<i64>,
    pub rcv: Option<String>,
    pub snd: Option<String>,
    pub note: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: String,
}

impl Transaction {
    pub fn get_memo(&self) -> Option<String> {
        self.note
            .clone()
            .and_then(|note| general_purpose::STANDARD.decode(note).ok())
            .and_then(|decoded| str::from_utf8(&decoded).ok().map(|s| s.to_string()))
            .map(|s| s.to_string())
    }
}

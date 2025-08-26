use core::str;

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsParams {
    #[serde(rename = "last-round")]
    pub last_round: i64,
    #[serde(rename = "genesis-hash")]
    pub genesis_hash: String,
    #[serde(rename = "genesis-id")]
    pub genesis_id: String,
    #[serde(rename = "min-fee")]
    pub min_fee: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    #[serde(rename = "round-time")]
    pub round_time: i64,
    pub fee: Option<i64>,
    pub sender: Option<String>,
    pub note: Option<String>,
    #[serde(rename = "payment-transaction")]
    pub payment_transaction: Option<PaymentTransaction>,
    #[serde(rename = "tx-type")]
    pub transaction_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    pub amount: Option<i64>,
    pub receiver: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBroadcast {
    #[serde(rename = "txId")]
    pub tx_id: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    #[serde(rename = "confirmed-round")]
    pub confirmed_round: Option<i64>,
}
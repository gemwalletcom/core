use serde::{Deserialize, Serialize};

use super::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    #[serde(rename = "blockHeight")]
    pub block_height: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinTransactionBroacastResult {
    pub error: Option<BitcoinTransactionBroacastError>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinTransactionBroacastError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinUTXO {
    pub txid: String,
    pub vout: i32,
    pub value: String,
}

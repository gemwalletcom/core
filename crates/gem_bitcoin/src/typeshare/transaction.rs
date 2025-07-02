use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct BitcoinTransaction {
    #[serde(rename = "blockHeight")]
    pub block_height: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct BitcoinTransactionBroacastResult {
    pub error: Option<BitcoinTransactionBroacastError>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct BitcoinTransactionBroacastError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct BitcoinUTXO {
    pub txid: String,
    pub vout: i32,
    pub value: String,
}

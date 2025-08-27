use serde::{Deserialize, Serialize};

use super::UInt64;

// Domain models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinTransaction {
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

// RPC models
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddressDetails {
    pub transactions: Option<Vec<Transaction>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub txid: String,
    pub value: String,
    pub value_in: String,
    pub fees: String,
    pub block_time: i64,
    pub block_height: i64,
    pub vin: Vec<Input>,
    pub vout: Vec<Output>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub is_address: bool,
    pub addresses: Option<Vec<String>>, // will be optional for Coinbase Input
    pub value: String,
    pub n: i64,
    pub tx_id: Option<String>, // will be optional for Coinbase Input
    pub vout: Option<i64>,     // will be optional for Coinbase Input
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub is_address: bool,
    pub addresses: Option<Vec<String>>,
    pub value: String,
    pub n: i64,
}

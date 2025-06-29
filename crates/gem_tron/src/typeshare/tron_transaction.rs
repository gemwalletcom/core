use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronTransactionBroadcast {
    pub result: bool,
    pub txid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronTransactionBroadcastError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronTransaction {
    pub ret: Vec<TronTransactionContractRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronTransactionContractRef {
    #[serde(rename = "contractRet")]
    pub contract_ret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronTransactionReceipt {
    #[serde(rename = "blockNumber")]
    pub block_number: i32,
    pub fee: Option<i32>,
    pub result: Option<String>,
    pub receipt: Option<TronReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronReceipt {
    pub result: Option<String>,
}
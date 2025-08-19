use serde::{Deserialize, Serialize};

use super::{Int64, UInt64};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronTransactionBroadcast {
    pub result: bool,
    pub txid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronTransactionBroadcastError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronTransaction {
    pub ret: Vec<TronTransactionContractRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronTransactionContractRef {
    #[serde(rename = "contractRet")]
    pub contract_ret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronTransactionReceipt {
    #[serde(rename = "blockNumber")]
    pub block_number: UInt64,
    pub fee: Option<Int64>,
    pub result: Option<String>,
    pub receipt: Option<TronReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronReceipt {
    pub result: Option<String>,
}

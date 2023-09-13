use primitives::BigIntHex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub block_number: BigIntHex,
    pub from: String,
    // pub gas: String,
    // pub gas_price: String,
    // pub max_priority_fee_per_gas: Option<String>,
    // pub max_fee_per_gas: Option<String>,
    pub hash: String,
    pub input: String,
    pub nonce: BigIntHex,
    pub to: Option<String>,
    pub value: BigIntHex,
    // #[serde(rename = "type")]
    // pub transaction_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReciept {
    // pub from: String,
    pub gas_used: BigIntHex,
    pub effective_gas_price: BigIntHex,
    // pub gas_price: String,
    // pub max_priority_fee_per_gas: Option<String>,
    // pub max_fee_per_gas: Option<String>,
    // pub hash: String,
    // pub input: String,
    // pub nonce: String,
    // pub to: Option<String>,
    // pub value: String,
    pub status: String,
    // #[serde(rename = "type")]
    // pub transaction_type: String,
}
use num_bigint::BigUint;
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
    pub gas_used: BigIntHex,
    pub effective_gas_price: BigIntHex,
    pub l1_fee: Option<BigIntHex>,
    pub logs: Vec<Log>,
    pub status: String,
}

impl TransactionReciept {
    pub fn get_fee(&self) -> BigUint {
        let fee = self.gas_used.clone().value * self.effective_gas_price.clone().value;
        if let Some(l1_fee) = self.l1_fee.clone() {
            return fee + l1_fee.value;
        }
        fee
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Log {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
}

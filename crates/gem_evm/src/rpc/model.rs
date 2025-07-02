use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_hex_str, deserialize_biguint_from_option_hex_str, deserialize_u64_from_str_or_int};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transactions: Vec<Transaction>,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub timestamp: BigUint,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransactionsIds {
    pub transactions: Vec<String>,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub timestamp: BigUint,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub from: String,
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub gas: u64,
    // pub gas_price: String,
    // pub max_priority_fee_per_gas: Option<String>,
    // pub max_fee_per_gas: Option<String>,
    pub hash: String,
    pub input: String,
    pub to: Option<String>,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub value: BigUint,
    // #[serde(rename = "type")]
    // pub transaction_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReciept {
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub gas_used: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub effective_gas_price: BigUint,
    #[serde(default, deserialize_with = "deserialize_biguint_from_option_hex_str")]
    pub l1_fee: Option<BigUint>,
    pub logs: Vec<Log>,
    pub status: String,
    pub block_number: String,
}

impl TransactionReciept {
    pub fn get_fee(&self) -> BigUint {
        let fee = self.gas_used.clone() * self.effective_gas_price.clone();
        if let Some(l1_fee) = self.l1_fee.clone() {
            return fee + l1_fee;
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReplayTrace {
    pub output: String,
    #[serde(default)]
    pub state_diff: HashMap<String, StateChange>,
    pub transaction_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChange {
    pub balance: Diff<String>,
    pub storage: HashMap<String, Diff<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Diff<T> {
    Change(Change<T>),
    Add(Add<T>),
    Delete(Delete<T>),
    Keep(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Change<T> {
    #[serde(rename = "*")]
    pub from_to: FromTo<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Add<T> {
    #[serde(rename = "+")]
    pub value: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Delete<T> {
    #[serde(rename = "-")]
    pub value: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FromTo<T> {
    pub from: T,
    pub to: T,
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::types::JsonRpcResponse;

    use super::*;

    #[test]
    fn test_decode_trace_replay_transaction() {
        let json_str = include_str!("../../tests/data/trace_replay_tx_trace.json");
        let trace_replay_transaction = serde_json::from_str::<JsonRpcResponse<TransactionReplayTrace>>(json_str).unwrap().result;

        assert_eq!(
            trace_replay_transaction.output,
            "0x00000000000000000000000000000000000000000000000002442b58bef3a87300000000000000000000000000000000000000000000002a48acab6204b00000"
        );
    }
}

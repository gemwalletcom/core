use crate::address::serializer::deserialize as tron_address_deserialize;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub mod account;
pub mod block;
pub mod chain;
pub mod contract;
pub mod transaction;

pub use account::*;
pub use block::*;
pub use chain::*;
pub use contract::*;
pub use transaction::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    pub block_header: BlockHeader,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockTransactions {
    pub block_header: BlockHeader,
    #[serde(default)]
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockHeader {
    pub raw_data: BlockHeaderData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockHeaderData {
    pub number: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    #[serde(rename = "txID")]
    pub tx_id: String,
    pub ret: Vec<ContractRet>,
    pub raw_data: TransactionData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractRet {
    #[serde(rename = "contractRet")]
    pub contract_ret: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionData {
    pub contract: Vec<Contract>,
    pub fee_limit: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Contract {
    #[serde(rename = "type")]
    pub contract_type: String,
    pub parameter: ContractParameter,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractParameter {
    pub type_url: String,
    pub value: ContractParameterValue,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractParameterValue {
    pub amount: Option<i64>,
    #[serde(default, deserialize_with = "tron_address_deserialize")]
    pub owner_address: Option<String>,
    #[serde(default, deserialize_with = "tron_address_deserialize")]
    pub to_address: Option<String>,
    #[serde(default, deserialize_with = "tron_address_deserialize")]
    pub contract_address: Option<String>,
    pub data: Option<String>,
    pub frozen_balance: Option<i64>,
    pub unfreeze_balance: Option<i64>,
    pub resource: Option<String>,
    pub votes: Option<Vec<VoteInfo>>,
    pub support: Option<bool>,
    pub call_value: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VoteInfo {
    pub vote_address: String,
    pub vote_count: i64,
}

pub type BlockTransactionsInfo = Vec<TransactionReceiptData>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionReceiptData {
    pub id: String,
    pub fee: Option<i64>,
    #[serde(rename = "blockNumber")]
    pub block_number: i64,
    #[serde(rename = "blockTimeStamp")]
    pub block_time_stamp: i64,
    pub receipt: TransactionReceipt,
    pub log: Option<Vec<TronLog>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionReceipt {
    pub result: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TronLog {
    pub topics: Option<Vec<String>>,
    pub data: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct TriggerConstantContractRequest {
    pub owner_address: String,
    pub contract_address: String,
    pub function_selector: String,
    pub parameter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_value: Option<u64>,
    pub visible: bool,
}

#[derive(Deserialize, Debug)]
pub struct TriggerConstantContractResponse {
    #[serde(default)]
    pub constant_result: Vec<String>,
    pub result: Option<TriggerContractResult>,
    pub energy_used: u64,
    #[serde(default)]
    pub energy_penalty: Option<u64>,
}

impl TriggerConstantContractResponse {
    pub fn get_energy(&self) -> Result<u64, TronRpcError> {
        if let Some(error) = self.result.as_ref().and_then(|r| r.check_error()) {
            return Err(error);
        }
        Ok(self.energy_used + self.energy_penalty.unwrap_or_default())
    }
}

#[derive(Deserialize, Debug)]
pub struct TriggerContractResult {
    pub result: Option<bool>,
    pub code: Option<String>,
    pub message: Option<String>,
}

impl TriggerContractResult {
    pub fn check_error(&self) -> Option<TronRpcError> {
        if self.result.unwrap_or(false) {
            return None;
        }

        let message = self.message.as_deref().map(|message_hex| {
            hex::decode(message_hex)
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_else(|| message_hex.to_string())
        });

        Some(TronRpcError { code: self.code.clone(), message })
    }
}

#[derive(Debug, Clone)]
pub struct TronRpcError {
    pub code: Option<String>,
    pub message: Option<String>,
}

impl fmt::Display for TronRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tron RPC Error {} {}", self.code.as_deref().unwrap_or(""), self.message.as_deref().unwrap_or(""))
    }
}

impl Error for TronRpcError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessesList {
    pub witnesses: Vec<WitnessAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WitnessAccount {
    pub address: String,
    pub vote_count: Option<i64>,
    pub url: String,
    pub is_jobs: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainParametersResponse {
    pub chain_parameter: Vec<ChainParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainParameter {
    pub key: String,
    pub value: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronTransactionBroadcast {
    #[serde(rename = "txid")]
    pub txid: Option<String>,
    pub code: Option<String>,
    pub message: Option<String>,
}

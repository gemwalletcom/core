use super::constants::{STAKE_DEPOSIT_EVENT, STAKE_WITHDRAW_EVENT};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_u64_from_str, deserialize_option_u64_from_str};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub chain_id: i32,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub ledger_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub block_height: String,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: Option<String>,
    pub sender: Option<String>,
    pub success: bool,
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str")]
    pub gas_used: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str")]
    pub gas_unit_price: Option<u64>,
    pub events: Option<Vec<Event>>,
    #[serde(rename = "type", default)]
    pub transaction_type: Option<String>,
    pub sequence_number: Option<String>,
    #[serde(default, deserialize_with = "deserialize_u64_from_str")]
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub guid: Guid,
    pub data: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountData {
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guid {
    pub account_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Empty {}

impl Event {
    pub fn get_amount(&self) -> Option<String> {
        let data = self.data.clone()?;
        match self.event_type.as_str() {
            STAKE_WITHDRAW_EVENT | STAKE_DEPOSIT_EVENT => serde_json::from_value::<AmountData>(data).ok()?.amount,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPayload {
    pub function: String,
    pub type_arguments: Vec<String>,
    pub arguments: Vec<String>,
    #[serde(rename = "type")]
    pub payload_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource<T> {
    #[serde(rename = "type")]
    pub type_field: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub coin: Option<CoinData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinData {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinStore {
    pub coin: Coin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinInfo {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub sequence_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub hash: Option<String>,
    pub message: Option<String>,
    pub error_code: Option<String>,
    pub vm_error_code: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasFee {
    pub deprioritized_gas_estimate: u64,
    pub gas_estimate: u64,
    pub prioritized_gas_estimate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub sender: String,
    pub sequence_number: String,
    pub max_gas_amount: String,
    pub gas_unit_price: String,
    pub expiration_timestamp_secs: String,
    pub payload: TransactionPayload,
    pub signature: TransactionSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSignature {
    #[serde(rename = "type")]
    pub signature_type: String,
    pub public_key: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSimulation {
    pub expiration_timestamp_secs: String,
    pub gas_unit_price: String,
    pub max_gas_amount: String,
    pub payload: TransactionPayload,
    pub sender: String,
    pub sequence_number: String,
    pub signature: TransactionSignature,
}

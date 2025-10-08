use super::super::constants::{FUNGIBLE_ASSET_DEPOSIT_EVENT, FUNGIBLE_ASSET_WITHDRAW_EVENT, STAKE_DEPOSIT_EVENT, STAKE_WITHDRAW_EVENT};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_option_u64_from_str, deserialize_u64_from_str};

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
    pub payload: Option<TransactionPayload>,
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
pub struct DelegationPoolAddStakeData {
    pub pool_address: String,
    pub amount_added: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationPoolUnlockStakeData {
    pub pool_address: String,
    pub amount_unlocked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guid {
    pub account_address: String,
}

impl Event {
    pub fn get_amount(&self) -> Option<String> {
        let data = self.data.clone()?;
        match self.event_type.as_str() {
            STAKE_WITHDRAW_EVENT | STAKE_DEPOSIT_EVENT | FUNGIBLE_ASSET_WITHDRAW_EVENT | FUNGIBLE_ASSET_DEPOSIT_EVENT => {
                serde_json::from_value::<AmountData>(data).ok()?.amount
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPayload {
    pub function: Option<String>,
    #[serde(default)]
    pub type_arguments: Vec<String>,
    #[serde(default)]
    pub arguments: Vec<serde_json::Value>,
    #[serde(rename = "type")]
    pub payload_type: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub hash: Option<String>,
    pub message: Option<String>,
    pub error_code: Option<String>,
    pub vm_error_code: Option<u64>,
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
pub struct TransactionBroadcast {
    pub hash: String,
}

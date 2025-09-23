use base64::{Engine as _, engine::general_purpose};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::str;
use std::str::FromStr;

use super::message::{AuthInfo, Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastRequest {
    pub mode: String,
    pub tx_bytes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResponse {
    pub tx_response: Option<TransactionResult>,
    pub code: Option<i32>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub txhash: String,
    pub code: i32,
    pub raw_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseLegacy {
    pub tx_response: TransactionResult,
}

#[derive(Debug, Clone)]
pub struct TransactionDecode {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx: TransactionResponseTx,
    pub tx_response: TransactionResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsResponse {
    pub txs: Vec<TransactionResponseTx>,
    pub tx_responses: Vec<TransactionResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseTx {
    pub body: TransactionBody,
    pub auth_info: AuthInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBody {
    pub memo: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseData {
    pub code: i64,
    pub txhash: String,
    pub events: Vec<TransactionEvent>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub attributes: Vec<TransactionEventAtribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEventAtribute {
    pub key: String,
    pub value: Option<String>,
}

impl TransactionResponse {
    pub fn get_rewards_value(&self, denom: &str) -> Option<BigInt> {
        let attributes = self
            .tx_response
            .events
            .clone()
            .into_iter()
            .filter(|x| x.event_type == crate::constants::EVENTS_WITHDRAW_REWARDS_TYPE)
            .flat_map(|x| x.attributes)
            .collect::<Vec<_>>();

        //base64 decoding added for sei/celestia. This is a temporary solution until the issue is resolved in the cosmos-sdk
        let value = attributes
            .into_iter()
            .filter(|x| {
                if let Ok(value) = general_purpose::STANDARD.decode(x.key.clone()) {
                    str::from_utf8(&value).unwrap() == crate::constants::EVENTS_ATTRIBUTE_AMOUNT
                } else {
                    x.key == crate::constants::EVENTS_ATTRIBUTE_AMOUNT
                }
            })
            .map(|x| {
                let value = x.value.unwrap_or_default();
                let decoded_value;
                let str_value = if let Ok(decoded) = general_purpose::STANDARD.decode(value.clone()) {
                    decoded_value = decoded;
                    str::from_utf8(&decoded_value).unwrap_or_default()
                } else {
                    &value
                };
                str_value
                    .split(',')
                    .filter(|x| x.contains(denom))
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap_or(&"0")
                    .to_string()
                    .replace(denom, "")
            })
            .flat_map(|x| BigInt::from_str(&x).ok())
            .sum();
        Some(value)
    }
}

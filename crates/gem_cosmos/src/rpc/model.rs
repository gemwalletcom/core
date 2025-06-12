use std::str;
use std::str::FromStr;

use base64::{engine::general_purpose, Engine as _};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

pub const EVENTS_WITHDRAW_REWARDS_TYPE: &str = "withdraw_rewards";
pub const EVENTS_ATTRIBUTE_AMOUNT: &str = "amount";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub block: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub data: BlockData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub txs: Vec<String>,
}

// transaction

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx: TransactionResponseTx,
    pub tx_response: TransactionResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseTx {
    pub body: TransactionResponseBody,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseBody {
    pub messages: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseData {
    pub code: i64,
    pub height: String,
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
            .filter(|x| x.event_type == EVENTS_WITHDRAW_REWARDS_TYPE) // Corrected: super::model:: removed
            .flat_map(|x| x.attributes)
            .collect::<Vec<_>>();

        //base64 decoding added for sei/celestia. This is a temporary solution until the issue is resolved in the cosmos-sdk
        let value = attributes
            .into_iter()
            .filter(|x| {
                if let Ok(value) = general_purpose::STANDARD.decode(x.key.clone()) {
                    str::from_utf8(&value).unwrap() == EVENTS_ATTRIBUTE_AMOUNT
                } else {
                    x.key == EVENTS_ATTRIBUTE_AMOUNT
                }
            })
            .map(|x| {
                let value = x.value.unwrap_or_default();
                if let Ok(value) = general_purpose::STANDARD.decode(value.clone()).as_ref() {
                    str::from_utf8(value).unwrap_or_default()
                } else {
                    &value
                }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSend {
    pub from_address: String,
    pub to_address: String,
    pub amount: Vec<Coin>,
}

impl MessageSend {
    pub fn get_amount(&self, denom: &str) -> Option<BigInt> {
        let value = self
            .amount
            .clone()
            .into_iter()
            .filter(|x| x.denom == denom)
            .flat_map(|x| BigInt::from_str(&x.amount).ok())
            .sum();
        Some(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    pub denom: String,
    pub amount: String,
}

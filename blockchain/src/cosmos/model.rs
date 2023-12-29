use std::str::FromStr;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

pub const EVENTS_WITHDRAW_REWARDS_TYPE: &str = "withdraw_rewards";

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
    pub tx_response: TransactionResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseData {
    pub code: i64,
    pub height: String,
    pub txhash: String,
    pub logs: Vec<TransactionResponseLogs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseLogs {
    pub events: Vec<TransactionEvent>,
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
    pub value: String,
}

impl TransactionResponse {
    pub fn get_rewards_value(&self, denom: &str) -> Option<BigInt> {
        let attbibutes = self
            .tx_response
            .logs
            .clone()
            .into_iter()
            .flat_map(|x| {
                x.events
                    .into_iter()
                    .filter(|x| x.event_type == super::model::EVENTS_WITHDRAW_REWARDS_TYPE)
            })
            .flat_map(|x| x.attributes)
            .collect::<Vec<_>>();

        let value = attbibutes
            .into_iter()
            .filter(|x| x.key == "amount")
            .map(|x| x.value.to_string().replace(denom, ""))
            .flat_map(|x| BigInt::from_str(&x).ok())
            .sum();
        Some(value)
    }
}

use std::str;
use std::str::FromStr;

use base64::{engine::general_purpose, Engine as _};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use crate::rpc::message::{AuthInfo, CosmosMessage, MsgSend};

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
    pub messages: Vec<CosmosMessage>,
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
            .filter(|x| x.event_type == EVENTS_WITHDRAW_REWARDS_TYPE)
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

impl MsgSend {
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
pub struct ValidatorsResponse {
    pub validators: Vec<Validator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub operator_address: String,
    pub jailed: bool,
    pub status: String,
    pub description: ValidatorDescription,
    pub commission: ValidatorCommission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorDescription {
    pub moniker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCommission {
    pub commission_rates: ValidatorCommissionRates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCommissionRates {
    pub rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPoolResponse {
    pub pool: StakingPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub bonded_tokens: String,
    pub not_bonded_tokens: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflationResponse {
    pub inflation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualProvisionsResponse {
    pub annual_provisions: String,
}

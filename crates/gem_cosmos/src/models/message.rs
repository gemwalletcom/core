use serde::{Deserialize, Serialize};

use crate::constants;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum Message {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend", alias = "/types.MsgSend")]
    MsgSend(MsgSend),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgUndelegate")]
    MsgUndelegate(MsgUndelegate),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgBeginRedelegate")]
    MsgBeginRedelegate(MsgBeginRedelegate),
    #[serde(rename = "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward")]
    MsgWithdrawDelegatorReward(MsgWithdrawDelegatorReward),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgDelegate")]
    MsgDelegate(MsgDelegate),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgSend {
    pub from_address: String,
    pub to_address: String,
    pub amount: Vec<Coin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub fee: Fee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coin {
    pub denom: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    pub amount: Vec<Coin>,
    pub gas_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgDelegate {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: Option<Coin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgUndelegate {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: Option<Coin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgBeginRedelegate {
    pub delegator_address: String,
    pub validator_src_address: String,
    pub validator_dst_address: String,
    pub amount: Option<Coin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgWithdrawDelegatorReward {
    pub delegator_address: String,
    pub validator_address: String,
}

impl Message {
    pub fn supported_types() -> &'static [&'static str] {
        constants::SUPPORTED_MESSAGES
    }
}

impl MsgSend {
    pub fn get_amount(&self, denom: &str) -> Option<num_bigint::BigInt> {
        use std::str::FromStr;
        let value = self
            .amount
            .clone()
            .into_iter()
            .filter(|x| x.denom == denom)
            .flat_map(|x| num_bigint::BigInt::from_str(&x.amount).ok())
            .sum();
        Some(value)
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum CosmosMessage {
    #[serde(rename = "/cosmos.bank.v1beta1.MsgSend")]
    MsgSend(MsgSend),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgUndelegate")]
    MsgUndelegate(MsgUndelegate),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgBeginRedelegate")]
    MsgBeginRedelegate(MsgBeginRedelegate),
    #[serde(rename = "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward")]
    MsgWithdrawDelegatorReward(MsgWithdrawDelegatorReward),
    #[serde(rename = "/cosmos.staking.v1beta1.MsgDelegate")]
    MsgDelegate(MsgDelegate),
    Unknown(serde_json::Value),
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

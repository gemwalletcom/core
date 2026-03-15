use serde::{Deserialize, Serialize};

use crate::constants;
#[cfg(feature = "signer")]
use crate::constants::{MESSAGE_EXECUTE_CONTRACT, MESSAGE_IBC_TRANSFER};
#[cfg(feature = "signer")]
use super::{ExecuteContractValue, IbcTransferValue};

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

#[cfg(feature = "signer")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageEnvelope {
    pub type_url: String,
    pub value: serde_json::Value,
}

#[cfg(feature = "signer")]
pub enum CosmosMessage {
    ExecuteContract {
        sender: String,
        contract: String,
        msg: Vec<u8>,
        funds: Vec<Coin>,
    },
    IbcTransfer {
        source_port: String,
        source_channel: String,
        token: Coin,
        sender: String,
        receiver: String,
        timeout_timestamp: u64,
        memo: String,
    },
}

#[cfg(feature = "signer")]
impl CosmosMessage {
    pub fn parse(data: &str) -> Result<Self, String> {
        let envelope: MessageEnvelope = serde_json::from_str(data).map_err(|e| format!("invalid swap data JSON: {e}"))?;

        match envelope.type_url.as_str() {
            MESSAGE_EXECUTE_CONTRACT => {
                let v: ExecuteContractValue = serde_json::from_value(envelope.value).map_err(|e| format!("invalid MsgExecuteContract: {e}"))?;
                Ok(Self::ExecuteContract {
                    sender: v.sender,
                    contract: v.contract,
                    msg: v.msg.into_bytes(),
                    funds: v.funds,
                })
            }
            MESSAGE_IBC_TRANSFER => {
                let v: IbcTransferValue = serde_json::from_value(envelope.value).map_err(|e| format!("invalid MsgTransfer: {e}"))?;
                Ok(Self::IbcTransfer {
                    source_port: v.source_port,
                    source_channel: v.source_channel,
                    token: v.token,
                    sender: v.sender,
                    receiver: v.receiver,
                    timeout_timestamp: v.timeout_timestamp,
                    memo: v.memo,
                })
            }
            other => Err(format!("unsupported cosmos message type: {other}")),
        }
    }
}

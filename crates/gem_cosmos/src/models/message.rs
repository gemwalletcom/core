use std::str::FromStr;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[cfg(feature = "signer")]
use super::{ExecuteContractValue, IbcTransferValue};
use crate::constants;
#[cfg(feature = "signer")]
use crate::constants::{
    MESSAGE_DELEGATE, MESSAGE_EXECUTE_CONTRACT, MESSAGE_IBC_TRANSFER, MESSAGE_REDELEGATE, MESSAGE_REWARD_BETA, MESSAGE_SEND, MESSAGE_SEND_BETA, MESSAGE_UNDELEGATE,
};
#[cfg(feature = "signer")]
use primitives::SignerError;

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

impl Coin {
    pub fn get_amount(&self) -> Option<BigInt> {
        BigInt::from_str(&self.amount).ok()
    }
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
    pub fn get_amount(&self, denom: &str) -> Option<BigInt> {
        Some(self.amount.iter().filter(|c| c.denom == denom).flat_map(Coin::get_amount).sum())
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
    Send {
        from_address: String,
        to_address: String,
        amount: Vec<Coin>,
    },
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
    Delegate {
        delegator_address: String,
        validator_address: String,
        amount: Coin,
    },
    Undelegate {
        delegator_address: String,
        validator_address: String,
        amount: Coin,
    },
    BeginRedelegate {
        delegator_address: String,
        validator_src_address: String,
        validator_dst_address: String,
        amount: Coin,
    },
    WithdrawDelegatorReward {
        delegator_address: String,
        validator_address: String,
    },
}

pub fn send_msg_json(from: &str, to: &str, denom: &str, amount: &str) -> serde_json::Value {
    serde_json::json!({
        "typeUrl": constants::MESSAGE_SEND_BETA,
        "value": {
            "from_address": from,
            "to_address": to,
            "amount": [{"denom": denom, "amount": amount}]
        }
    })
}

#[cfg(feature = "signer")]
impl CosmosMessage {
    pub fn parse_array(data: &str) -> Result<Vec<Self>, SignerError> {
        let arr: Vec<serde_json::Value> = serde_json::from_str(data)?;
        arr.iter().map(|v| Self::parse(&v.to_string())).collect()
    }

    pub fn parse(data: &str) -> Result<Self, SignerError> {
        let envelope: MessageEnvelope = serde_json::from_str(data)?;

        match envelope.type_url.as_str() {
            MESSAGE_SEND_BETA | MESSAGE_SEND => {
                let v: MsgSend = serde_json::from_value(envelope.value)?;
                Ok(Self::Send {
                    from_address: v.from_address,
                    to_address: v.to_address,
                    amount: v.amount,
                })
            }
            MESSAGE_EXECUTE_CONTRACT => {
                let v: ExecuteContractValue = serde_json::from_value(envelope.value)?;
                Ok(Self::ExecuteContract {
                    sender: v.sender,
                    contract: v.contract,
                    msg: v.msg.into_bytes(),
                    funds: v.funds,
                })
            }
            MESSAGE_IBC_TRANSFER => {
                let v: IbcTransferValue = serde_json::from_value(envelope.value)?;
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
            MESSAGE_DELEGATE => {
                let v: MsgDelegate = serde_json::from_value(envelope.value)?;
                let amount = v.amount.ok_or_else(|| SignerError::invalid_input("missing delegate amount"))?;
                Ok(Self::Delegate {
                    delegator_address: v.delegator_address,
                    validator_address: v.validator_address,
                    amount,
                })
            }
            MESSAGE_UNDELEGATE => {
                let v: MsgUndelegate = serde_json::from_value(envelope.value)?;
                let amount = v.amount.ok_or_else(|| SignerError::invalid_input("missing undelegate amount"))?;
                Ok(Self::Undelegate {
                    delegator_address: v.delegator_address,
                    validator_address: v.validator_address,
                    amount,
                })
            }
            MESSAGE_REDELEGATE => {
                let v: MsgBeginRedelegate = serde_json::from_value(envelope.value)?;
                let amount = v.amount.ok_or_else(|| SignerError::invalid_input("missing redelegate amount"))?;
                Ok(Self::BeginRedelegate {
                    delegator_address: v.delegator_address,
                    validator_src_address: v.validator_src_address,
                    validator_dst_address: v.validator_dst_address,
                    amount,
                })
            }
            MESSAGE_REWARD_BETA => {
                let v: MsgWithdrawDelegatorReward = serde_json::from_value(envelope.value)?;
                Ok(Self::WithdrawDelegatorReward {
                    delegator_address: v.delegator_address,
                    validator_address: v.validator_address,
                })
            }
            other => SignerError::invalid_input_err(format!("unsupported cosmos message type: {other}")),
        }
    }
}

#[cfg(all(test, feature = "signer"))]
mod tests {
    use super::*;

    #[test]
    fn test_parse_execute_contract() {
        let msg = CosmosMessage::parse(include_str!("../../testdata/swap_execute_contract.json")).unwrap();
        match msg {
            CosmosMessage::ExecuteContract { sender, contract, funds, .. } => {
                assert_eq!(sender, "osmo1tkvyjqeq204rmrrz3w4hcrs336qahsfwn8m0ye");
                assert_eq!(contract, "osmo1n6ney9tsf55etz9nrmzyd8wa7e64qd3s06a74fqs30ka8pps6cvqtsycr6");
                assert_eq!(funds.len(), 1);
                assert_eq!(funds[0].denom, "uosmo");
                assert_eq!(funds[0].amount, "10000000");
            }
            _ => panic!("expected ExecuteContract"),
        }
    }

    #[test]
    fn test_parse_ibc_transfer() {
        let msg = CosmosMessage::parse(include_str!("../../testdata/swap_ibc_transfer.json")).unwrap();
        match msg {
            CosmosMessage::IbcTransfer {
                source_port,
                source_channel,
                sender,
                receiver,
                timeout_timestamp,
                memo,
                ..
            } => {
                assert_eq!(source_port, "transfer");
                assert_eq!(source_channel, "channel-141");
                assert_eq!(sender, "cosmos1tkvyjqeq204rmrrz3w4hcrs336qahsfwmugljt");
                assert_eq!(receiver, "osmo1n6ney9tsf55etz9nrmzyd8wa7e64qd3s06a74fqs30ka8pps6cvqtsycr6");
                assert_eq!(timeout_timestamp, 1773632858715000064);
                assert!(!memo.is_empty());
            }
            _ => panic!("expected IbcTransfer"),
        }
    }
}

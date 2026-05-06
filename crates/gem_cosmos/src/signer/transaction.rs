use gem_encoding::protobuf::*;
use primitives::{Address, DelegationValidator, SignerError, SignerInput, StakeType, chain_cosmos::CosmosChain};

use crate::address::CosmosAddress;
use crate::constants::{
    MESSAGE_DELEGATE, MESSAGE_EXECUTE_CONTRACT, MESSAGE_IBC_TRANSFER, MESSAGE_REDELEGATE, MESSAGE_REWARD_BETA, MESSAGE_SEND, MESSAGE_SEND_BETA, MESSAGE_UNDELEGATE,
};
use crate::models::{Coin, CosmosMessage};

pub const COSMOS_SECP256K1_PUBKEY_TYPE: &str = "/cosmos.crypto.secp256k1.PubKey";
pub const INJECTIVE_ETHSECP256K1_PUBKEY_TYPE: &str = "/injective.crypto.v1beta1.ethsecp256k1.PubKey";
const SIGN_MODE_DIRECT: u64 = 1;

pub fn transfer_message(input: &SignerInput, denom: &str) -> CosmosMessage {
    CosmosMessage::Send {
        from_address: input.sender_address.clone(),
        to_address: input.destination_address.clone(),
        amount: vec![Coin {
            denom: denom.to_string(),
            amount: input.value.clone(),
        }],
    }
}

pub fn stake_messages(input: &SignerInput, chain: CosmosChain) -> Result<Vec<CosmosMessage>, SignerError> {
    let stake_type = input.input_type.get_stake_type().map_err(SignerError::invalid_input)?;
    let delegator_address = &input.sender_address;
    let amount = Coin {
        denom: chain.denom().as_ref().to_string(),
        amount: input.value.clone(),
    };

    match stake_type {
        StakeType::Stake(validator) => Ok(vec![CosmosMessage::Delegate {
            delegator_address: delegator_address.clone(),
            validator_address: validator.id.clone(),
            amount,
        }]),
        StakeType::Unstake(delegation) => {
            let mut messages = reward_messages(delegator_address, std::slice::from_ref(&delegation.validator));
            messages.push(CosmosMessage::Undelegate {
                delegator_address: delegator_address.clone(),
                validator_address: delegation.validator.id.clone(),
                amount,
            });
            Ok(messages)
        }
        StakeType::Redelegate(data) => {
            let mut messages = reward_messages(delegator_address, std::slice::from_ref(&data.delegation.validator));
            messages.push(CosmosMessage::BeginRedelegate {
                delegator_address: delegator_address.clone(),
                validator_src_address: data.delegation.validator.id.clone(),
                validator_dst_address: data.to_validator.id.clone(),
                amount,
            });
            Ok(messages)
        }
        StakeType::Rewards(validators) => Ok(reward_messages(delegator_address, validators)),
        StakeType::Withdraw(_) => SignerError::invalid_input_err("Cosmos withdraw operations are not supported"),
        StakeType::Freeze(_) | StakeType::Unfreeze(_) => SignerError::invalid_input_err("Cosmos freeze operations are not supported"),
    }
}

fn encode_send(chain: CosmosChain, from_address: &str, to_address: &str, amount: &[Coin]) -> Result<Vec<u8>, SignerError> {
    let coin_fields: Vec<u8> = amount.iter().flat_map(|c| encode_message_field(3, &encode_coin(&c.denom, &c.amount))).collect();
    let address_fields = match chain {
        CosmosChain::Thorchain => {
            let parse = |addr: &str| CosmosAddress::try_parse(addr).ok_or_else(|| SignerError::invalid_input(format!("invalid cosmos address: {addr}")));
            [encode_bytes_field(1, parse(from_address)?.as_bytes()), encode_bytes_field(2, parse(to_address)?.as_bytes())].concat()
        }
        CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Injective | CosmosChain::Sei | CosmosChain::Noble => {
            [encode_string_field(1, from_address), encode_string_field(2, to_address)].concat()
        }
    };
    Ok([address_fields, coin_fields].concat())
}

fn encode_coin(denom: &str, amount: &str) -> Vec<u8> {
    [encode_string_field(1, denom), encode_string_field(2, amount)].concat()
}

fn reward_messages(delegator_address: &str, validators: &[DelegationValidator]) -> Vec<CosmosMessage> {
    validators
        .iter()
        .map(|validator| CosmosMessage::WithdrawDelegatorReward {
            delegator_address: delegator_address.to_string(),
            validator_address: validator.id.clone(),
        })
        .collect()
}

pub struct CosmosTxParams<'a> {
    pub body_bytes: Vec<u8>,
    pub chain_id: &'a str,
    pub account_number: u64,
    pub sequence: u64,
    pub fee_coins: Vec<Coin>,
    pub gas_limit: u64,
    pub pubkey_type: &'a str,
}

impl CosmosTxParams<'_> {
    pub fn encode_tx_body(messages: &[Vec<u8>], memo: &str) -> Vec<u8> {
        let msg_fields: Vec<u8> = messages.iter().flat_map(|m| encode_message_field(1, m)).collect();
        [msg_fields, encode_string_field(2, memo)].concat()
    }

    pub fn encode_auth_info(&self, pubkey_bytes: &[u8]) -> Vec<u8> {
        [
            encode_message_field(1, &Self::encode_signer_info(self.pubkey_type, pubkey_bytes, self.sequence)),
            encode_message_field(2, &Self::encode_fee(&self.fee_coins, self.gas_limit)),
        ]
        .concat()
    }

    pub fn encode_sign_doc(&self, body_bytes: &[u8], auth_info_bytes: &[u8]) -> Vec<u8> {
        [
            encode_bytes_field(1, body_bytes),
            encode_bytes_field(2, auth_info_bytes),
            encode_string_field(3, self.chain_id),
            encode_varint_field(4, self.account_number),
        ]
        .concat()
    }

    pub fn encode_tx_raw(body_bytes: &[u8], auth_info_bytes: &[u8], signature: &[u8]) -> Vec<u8> {
        [encode_bytes_field(1, body_bytes), encode_bytes_field(2, auth_info_bytes), encode_bytes_field(3, signature)].concat()
    }

    fn encode_pubkey_any(pubkey_type: &str, pubkey_bytes: &[u8]) -> Vec<u8> {
        [encode_string_field(1, pubkey_type), encode_bytes_field(2, &encode_bytes_field(1, pubkey_bytes))].concat()
    }

    fn encode_mode_info_single() -> Vec<u8> {
        encode_message_field(1, &encode_varint_field(1, SIGN_MODE_DIRECT))
    }

    fn encode_signer_info(pubkey_type: &str, pubkey_bytes: &[u8], sequence: u64) -> Vec<u8> {
        [
            encode_message_field(1, &Self::encode_pubkey_any(pubkey_type, pubkey_bytes)),
            encode_message_field(2, &Self::encode_mode_info_single()),
            encode_varint_field(3, sequence),
        ]
        .concat()
    }

    fn encode_fee(coins: &[Coin], gas_limit: u64) -> Vec<u8> {
        let coin_fields: Vec<u8> = coins.iter().flat_map(|c| encode_message_field(1, &encode_coin(&c.denom, &c.amount))).collect();
        [coin_fields, encode_varint_field(2, gas_limit)].concat()
    }
}

impl CosmosMessage {
    fn type_url(&self, chain: CosmosChain) -> &str {
        match self {
            Self::Send { .. } => match chain {
                CosmosChain::Thorchain => MESSAGE_SEND,
                CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Injective | CosmosChain::Sei | CosmosChain::Noble => MESSAGE_SEND_BETA,
            },
            Self::ExecuteContract { .. } => MESSAGE_EXECUTE_CONTRACT,
            Self::IbcTransfer { .. } => MESSAGE_IBC_TRANSFER,
            Self::Delegate { .. } => MESSAGE_DELEGATE,
            Self::Undelegate { .. } => MESSAGE_UNDELEGATE,
            Self::BeginRedelegate { .. } => MESSAGE_REDELEGATE,
            Self::WithdrawDelegatorReward { .. } => MESSAGE_REWARD_BETA,
        }
    }

    fn encode_value(&self, chain: CosmosChain) -> Result<Vec<u8>, SignerError> {
        match self {
            Self::Send { from_address, to_address, amount } => encode_send(chain, from_address, to_address, amount),
            Self::ExecuteContract { sender, contract, msg, funds } => {
                let fund_fields: Vec<u8> = funds.iter().flat_map(|c| encode_message_field(5, &encode_coin(&c.denom, &c.amount))).collect();
                Ok([encode_string_field(1, sender), encode_string_field(2, contract), encode_bytes_field(3, msg), fund_fields].concat())
            }
            Self::IbcTransfer {
                source_port,
                source_channel,
                token,
                sender,
                receiver,
                timeout_timestamp,
                memo,
            } => Ok([
                encode_string_field(1, source_port),
                encode_string_field(2, source_channel),
                encode_message_field(3, &encode_coin(&token.denom, &token.amount)),
                encode_string_field(4, sender),
                encode_string_field(5, receiver),
                encode_varint_field(7, *timeout_timestamp),
                encode_string_field(8, memo),
            ]
            .concat()),
            Self::Delegate {
                delegator_address,
                validator_address,
                amount,
            }
            | Self::Undelegate {
                delegator_address,
                validator_address,
                amount,
            } => Ok([
                encode_string_field(1, delegator_address),
                encode_string_field(2, validator_address),
                encode_message_field(3, &encode_coin(&amount.denom, &amount.amount)),
            ]
            .concat()),
            Self::BeginRedelegate {
                delegator_address,
                validator_src_address,
                validator_dst_address,
                amount,
            } => Ok([
                encode_string_field(1, delegator_address),
                encode_string_field(2, validator_src_address),
                encode_string_field(3, validator_dst_address),
                encode_message_field(4, &encode_coin(&amount.denom, &amount.amount)),
            ]
            .concat()),
            Self::WithdrawDelegatorReward {
                delegator_address,
                validator_address,
            } => Ok([encode_string_field(1, delegator_address), encode_string_field(2, validator_address)].concat()),
        }
    }

    pub fn encode_as_any(&self, chain: CosmosChain) -> Result<Vec<u8>, SignerError> {
        Ok([encode_string_field(1, self.type_url(chain)), encode_bytes_field(2, &self.encode_value(chain)?)].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_execute_contract() {
        let msg = CosmosMessage::ExecuteContract {
            sender: "osmo1test".to_string(),
            contract: "osmo1contract".to_string(),
            msg: b"{\"swap\":{}}".to_vec(),
            funds: vec![Coin {
                denom: "uosmo".to_string(),
                amount: "1000000".to_string(),
            }],
        };
        assert_eq!(
            hex::encode(msg.encode_as_any(CosmosChain::Osmosis).unwrap()),
            "0a242f636f736d7761736d2e7761736d2e76312e4d736745786563757465436f6e747261637412390a096f736d6f3174657374120d6f736d6f31636f6e74726163741a0b7b2273776170223a7b7d7d2a100a05756f736d6f120731303030303030"
        );
    }

    #[test]
    fn test_encode_ibc_transfer() {
        let msg = CosmosMessage::IbcTransfer {
            source_port: "transfer".to_string(),
            source_channel: "channel-0".to_string(),
            token: Coin {
                denom: "uatom".to_string(),
                amount: "1000000".to_string(),
            },
            sender: "cosmos1test".to_string(),
            receiver: "osmo1test".to_string(),
            timeout_timestamp: 1773382733549000000,
            memo: "{\"ibc_callback\":\"osmo1contract\"}".to_string(),
        };
        assert_eq!(
            hex::encode(msg.encode_as_any(CosmosChain::Cosmos).unwrap()),
            "0a292f6962632e6170706c69636174696f6e732e7472616e736665722e76312e4d73675472616e73666572126b0a087472616e7366657212096368616e6e656c2d301a100a057561746f6d120731303030303030220b636f736d6f7331746573742a096f736d6f317465737438c0aaffdfb4c694ce1842207b226962635f63616c6c6261636b223a226f736d6f31636f6e7472616374227d"
        );
    }
}

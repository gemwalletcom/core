use crate::models::{Coin, CosmosMessage};

use crate::constants::{MESSAGE_EXECUTE_CONTRACT, MESSAGE_IBC_TRANSFER};

use super::protobuf::*;

const MESSAGE_SEND_TYPE_URL: &str = "/cosmos.bank.v1beta1.MsgSend";
pub const COSMOS_SECP256K1_PUBKEY_TYPE: &str = "/cosmos.crypto.secp256k1.PubKey";
pub const INJECTIVE_ETHSECP256K1_PUBKEY_TYPE: &str = "/injective.crypto.v1beta1.ethsecp256k1.PubKey";
const SIGN_MODE_DIRECT: u64 = 1;

pub struct CosmosTxParams<'a> {
    pub body_bytes: Vec<u8>,
    pub chain_id: &'a str,
    pub account_number: u64,
    pub sequence: u64,
    pub fee_coins: Vec<Coin>,
    pub gas_limit: u64,
    pub pubkey_type: &'a str,
}

impl CosmosMessage {
    fn type_url(&self) -> &str {
        match self {
            Self::Send { .. } => MESSAGE_SEND_TYPE_URL,
            Self::ExecuteContract { .. } => MESSAGE_EXECUTE_CONTRACT,
            Self::IbcTransfer { .. } => MESSAGE_IBC_TRANSFER,
        }
    }

    fn encode_value(&self) -> Vec<u8> {
        match self {
            Self::Send { from_address, to_address, amount } => {
                let coin_fields: Vec<u8> = amount.iter().flat_map(|c| encode_message_field(3, &encode_coin(&c.denom, &c.amount))).collect();
                [encode_string_field(1, from_address), encode_string_field(2, to_address), coin_fields].concat()
            }
            Self::ExecuteContract { sender, contract, msg, funds } => {
                let fund_fields: Vec<u8> = funds.iter().flat_map(|c| encode_message_field(5, &encode_coin(&c.denom, &c.amount))).collect();
                [encode_string_field(1, sender), encode_string_field(2, contract), encode_bytes_field(3, msg), fund_fields].concat()
            }
            Self::IbcTransfer {
                source_port,
                source_channel,
                token,
                sender,
                receiver,
                timeout_timestamp,
                memo,
            } => [
                encode_string_field(1, source_port),
                encode_string_field(2, source_channel),
                encode_message_field(3, &encode_coin(&token.denom, &token.amount)),
                encode_string_field(4, sender),
                encode_string_field(5, receiver),
                // field number skips 6
                encode_varint_field(7, *timeout_timestamp),
                encode_string_field(8, memo),
            ]
            .concat(),
        }
    }

    pub fn encode_as_any(&self) -> Vec<u8> {
        [encode_string_field(1, self.type_url()), encode_bytes_field(2, &self.encode_value())].concat()
    }
}

pub fn encode_tx_body(messages: &[Vec<u8>], memo: &str) -> Vec<u8> {
    let msg_fields: Vec<u8> = messages.iter().flat_map(|m| encode_message_field(1, m)).collect();
    [msg_fields, encode_string_field(2, memo)].concat()
}

fn encode_pubkey_any(pubkey_type: &str, pubkey_bytes: &[u8]) -> Vec<u8> {
    [encode_string_field(1, pubkey_type), encode_bytes_field(2, &encode_bytes_field(1, pubkey_bytes))].concat()
}

fn encode_mode_info_single() -> Vec<u8> {
    encode_message_field(1, &encode_varint_field(1, SIGN_MODE_DIRECT))
}

fn encode_signer_info(pubkey_type: &str, pubkey_bytes: &[u8], sequence: u64) -> Vec<u8> {
    [
        encode_message_field(1, &encode_pubkey_any(pubkey_type, pubkey_bytes)),
        encode_message_field(2, &encode_mode_info_single()),
        encode_varint_field(3, sequence),
    ]
    .concat()
}

fn encode_fee(coins: &[Coin], gas_limit: u64) -> Vec<u8> {
    let coin_fields: Vec<u8> = coins.iter().flat_map(|c| encode_message_field(1, &encode_coin(&c.denom, &c.amount))).collect();
    [coin_fields, encode_varint_field(2, gas_limit)].concat()
}

pub fn encode_auth_info(pubkey_type: &str, pubkey_bytes: &[u8], sequence: u64, fee_coins: &[Coin], gas_limit: u64) -> Vec<u8> {
    [
        encode_message_field(1, &encode_signer_info(pubkey_type, pubkey_bytes, sequence)),
        encode_message_field(2, &encode_fee(fee_coins, gas_limit)),
    ]
    .concat()
}

pub fn encode_sign_doc(body_bytes: &[u8], auth_info_bytes: &[u8], chain_id: &str, account_number: u64) -> Vec<u8> {
    [
        encode_bytes_field(1, body_bytes),
        encode_bytes_field(2, auth_info_bytes),
        encode_string_field(3, chain_id),
        encode_varint_field(4, account_number),
    ]
    .concat()
}

pub fn encode_tx_raw(body_bytes: &[u8], auth_info_bytes: &[u8], signature: &[u8]) -> Vec<u8> {
    [encode_bytes_field(1, body_bytes), encode_bytes_field(2, auth_info_bytes), encode_bytes_field(3, signature)].concat()
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
            hex::encode(msg.encode_as_any()),
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
            hex::encode(msg.encode_as_any()),
            "0a292f6962632e6170706c69636174696f6e732e7472616e736665722e76312e4d73675472616e73666572126b0a087472616e7366657212096368616e6e656c2d301a100a057561746f6d120731303030303030220b636f736d6f7331746573742a096f736d6f317465737438c0aaffdfb4c694ce1842207b226962635f63616c6c6261636b223a226f736d6f31636f6e7472616374227d"
        );
    }
}

use base64::{Engine as _, engine::general_purpose};
use chrono::DateTime;
use primitives::{AssetId, Chain, StakeValidator, Transaction, TransactionState, TransactionType, TransactionUpdate};
use sha2::{Digest, Sha256};
use std::error::Error;

use crate::models::BroadcastResponse;
use crate::models::{AuthInfo, Message, TransactionBody, TransactionResponse, Validator};

pub fn map_transaction_broadcast(response: &BroadcastResponse) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(tx_response) = &response.tx_response {
        if tx_response.code != 0 {
            Err(tx_response.raw_log.clone().into())
        } else {
            Ok(tx_response.txhash.clone())
        }
    } else if let Some(message) = &response.message {
        Err(format!("Broadcast error: {}", message).into())
    } else {
        Err("Unknown broadcast error".into())
    }
}

pub fn map_transaction_status(transaction: TransactionResponse) -> TransactionUpdate {
    let state = if transaction.tx_response.code == 0 {
        TransactionState::Confirmed
    } else {
        TransactionState::Reverted
    };

    TransactionUpdate::new_state(state)
}

pub fn map_transaction_decode(body: String) -> Option<String> {
    let bytes = general_purpose::STANDARD.decode(body.clone()).ok()?;
    Some(get_hash(bytes))
}

pub fn get_hash(bytes: Vec<u8>) -> String {
    hex::encode(Sha256::digest(bytes.clone())).to_uppercase()
}

pub fn map_transactions(chain: Chain, transactions: Vec<TransactionResponse>) -> Vec<primitives::Transaction> {
    transactions
        .clone()
        .into_iter()
        .filter_map(|x| map_transaction(chain, x.tx.body.clone(), x.tx.auth_info.clone(), x.clone()))
        .collect::<Vec<Transaction>>()
}

pub fn map_transaction(chain: Chain, body: TransactionBody, auth_info: Option<AuthInfo>, transaction: TransactionResponse) -> Option<Transaction> {
    let hash = transaction.tx_response.txhash.clone();
    let default_denom = chain.as_denom()?;
    let fee = auth_info?.fee.amount.into_iter().filter(|x| x.denom == default_denom).collect::<Vec<_>>();
    let fee = fee.first()?.amount.clone();
    let memo = if body.memo.is_empty() { None } else { Some(body.memo.clone()) };

    let state = if transaction.tx_response.code == 0 {
        TransactionState::Confirmed
    } else {
        TransactionState::Reverted
    };
    let created_at = DateTime::parse_from_rfc3339(&transaction.tx_response.timestamp).ok()?.into();

    if body.messages.len() > 1 {
        return None;
    }

    for message in body.messages {
        let asset_id: AssetId;
        let transaction_type: TransactionType;
        let value: String;
        let from_address: String;
        let to_address: String;
        match message {
            Message::MsgSend(message) => {
                asset_id = chain.as_asset_id();
                transaction_type = TransactionType::Transfer;
                value = message.amount.first()?.amount.clone();
                from_address = message.from_address;
                to_address = message.to_address;
            }
            Message::MsgDelegate(message) => {
                asset_id = chain.as_asset_id();
                transaction_type = TransactionType::StakeDelegate;
                value = message.amount?.amount.clone();
                from_address = message.delegator_address;
                to_address = message.validator_address;
            }
            Message::MsgUndelegate(message) => {
                asset_id = chain.as_asset_id();
                transaction_type = TransactionType::StakeUndelegate;
                value = message.amount?.amount.clone();
                from_address = message.delegator_address;
                to_address = message.validator_address;
            }
            Message::MsgBeginRedelegate(message) => {
                asset_id = chain.as_asset_id();
                transaction_type = TransactionType::StakeRedelegate;
                value = message.amount?.amount.clone();
                from_address = message.delegator_address;
                to_address = message.validator_dst_address;
            }
            Message::MsgWithdrawDelegatorReward(message) => {
                asset_id = chain.as_asset_id();
                value = transaction.get_rewards_value(chain.as_denom()?)?.to_string();
                transaction_type = TransactionType::StakeRewards;
                from_address = message.delegator_address;
                to_address = message.validator_address;
            }
            _ => {
                continue;
            }
        }

        let transaction = Transaction::new(
            hash,
            asset_id.clone(),
            from_address,
            to_address,
            None,
            transaction_type,
            state,
            fee,
            asset_id.clone(),
            value,
            memo,
            None,
            created_at,
        );
        return Some(transaction);
    }
    None
}

pub fn map_validators(validators: Vec<Validator>) -> Vec<StakeValidator> {
    validators
        .into_iter()
        .map(|v| StakeValidator::new(v.operator_address, v.description.moniker))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{BroadcastResponse, TransactionResult};

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response = BroadcastResponse {
            tx_response: Some(TransactionResult {
                txhash: "ABC123".to_string(),
                code: 0,
                raw_log: "".to_string(),
            }),
            code: None,
            message: None,
        };

        assert_eq!(map_transaction_broadcast(&response).unwrap(), "ABC123");
    }

    #[test]
    fn test_map_transaction_broadcast_failed() {
        let response: BroadcastResponse = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_failed.json")).unwrap();

        assert!(map_transaction_broadcast(&response).is_err());
    }

    #[test]
    fn test_map_transaction_failed() {
        let response: BroadcastResponse = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_failed.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "signature verification failed; please verify account number (1343971) and chain-id (cosmoshub-4): (unable to verify single signer signature): unauthorized"
        );
    }

    #[test]
    fn test_transfer() {
        let file_content = include_str!("../../testdata/transfer.json");
        let result: TransactionResponse = serde_json::from_str(file_content).unwrap();

        let transaction = map_transactions(Chain::Cosmos, vec![result]).first().unwrap().clone();
        let expected = Transaction::new(
            "BC5E330F0AFA34489B9796E8101A2B027CC8AE8E820AFC7901C3C1E75C2895DD".to_string(),
            Chain::Cosmos.as_asset_id(),
            "cosmos1wev8ptzj27aueu04wgvvl4gvurax6rj5f0v7rw".to_string(),
            "cosmos1hgp84me0lze8t4jfrwsr05aep2kr57zrk4gecx".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            "1600".to_string(),
            Chain::Cosmos.as_asset_id(),
            "50000000".to_string(),
            Some("6439432658467882".to_string()),
            None,
            DateTime::parse_from_rfc3339("2025-06-20T04:09:19Z").unwrap().into(),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_delegate() {
        let file_content = include_str!("../../testdata/delegate.json");
        let result: TransactionResponse = serde_json::from_str(file_content).unwrap();

        let transaction = map_transactions(Chain::Cosmos, vec![result]).first().unwrap().clone();
        let expected = Transaction::new(
            "FD334515F2D872B6689D7B52598796BF91C42111C857D6E80E984BC6DB4B0575".to_string(),
            Chain::Cosmos.as_asset_id(),
            "cosmos1z64xeecaqudhe2scx0m4mtvh7d0g5khyakpsmw".to_string(),
            "cosmosvaloper1jlr62guqwrwkdt4m3y00zh2rrsamhjf9num5xr".to_string(),
            None,
            TransactionType::StakeDelegate,
            TransactionState::Confirmed,
            "5194".to_string(),
            Chain::Cosmos.as_asset_id(),
            "17732657".to_string(),
            None,
            None,
            DateTime::parse_from_rfc3339("2025-06-21T20:33:42Z").unwrap().into(),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_rewards() {
        let file_content = include_str!("../../testdata/rewards.json");

        let result: TransactionResponse = serde_json::from_str(file_content).unwrap();

        let transaction = map_transactions(Chain::Cosmos, vec![result]).first().unwrap().clone();
        let expected = Transaction::new(
            "0B615F5DDDB216574DF8AC07B104C3C902B23974C7957DF4275E1572CDDAFCB4".to_string(),
            Chain::Cosmos.as_asset_id(),
            "cosmos1cvh8mpz04az0x7vht6h6ekksg8wd650r39ltwj".to_string(),
            "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3".to_string(),
            None,
            TransactionType::StakeRewards,
            TransactionState::Confirmed,
            "25000".to_string(),
            Chain::Cosmos.as_asset_id(),
            "2385518".to_string(),
            None,
            None,
            DateTime::parse_from_rfc3339("2025-06-21T20:51:28Z").unwrap().into(),
        );

        assert_eq!(transaction, expected);
    }
}

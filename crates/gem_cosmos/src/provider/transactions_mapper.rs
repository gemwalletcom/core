use base64::{Engine as _, engine::general_purpose};
use chrono::DateTime;
use primitives::chain_cosmos::CosmosChain;
use primitives::{AssetId, StakeValidator, Transaction, TransactionState, TransactionType};
use sha2::{Digest, Sha256};
use std::error::Error;

use crate::constants::get_base_fee;
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

pub fn map_transaction_decode(body: String) -> Option<String> {
    let bytes = general_purpose::STANDARD.decode(body.clone()).ok()?;
    let decoded_str = String::from_utf8_lossy(&bytes);
    let has_supported_type = crate::constants::SUPPORTED_MESSAGES.iter().any(|msg_type| decoded_str.contains(msg_type));
    if has_supported_type { Some(get_hash(bytes)) } else { None }
}

pub fn get_hash(bytes: Vec<u8>) -> String {
    hex::encode(Sha256::digest(bytes.clone())).to_uppercase()
}

pub fn map_transactions(chain: CosmosChain, transactions: Vec<TransactionResponse>) -> Vec<primitives::Transaction> {
    transactions
        .into_iter()
        .filter_map(|x| {
            let body = x.tx.body.clone();
            let auth_info = x.tx.auth_info.clone();
            map_transaction(chain, body, auth_info, x)
        })
        .collect()
}

pub fn map_transaction(chain: CosmosChain, body: TransactionBody, auth_info: Option<AuthInfo>, transaction: TransactionResponse) -> Option<Transaction> {
    let hash = transaction.tx_response.txhash.clone();
    let default_denom = chain.as_chain().as_denom()?.to_string();
    let fee = auth_info?.fee.amount.into_iter().filter(|x| x.denom == default_denom).collect::<Vec<_>>();
    let fee = fee.first().map(|f| f.amount.clone()).unwrap_or_else(|| get_base_fee(chain).to_string());
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
                asset_id = chain.as_chain().as_asset_id();
                transaction_type = TransactionType::Transfer;
                value = message.amount.first()?.amount.clone();
                from_address = message.from_address;
                to_address = message.to_address;
            }
            Message::MsgDelegate(message) => {
                asset_id = chain.as_chain().as_asset_id();
                transaction_type = TransactionType::StakeDelegate;
                value = message.amount?.amount.clone();
                from_address = message.delegator_address;
                to_address = message.validator_address;
            }
            Message::MsgUndelegate(message) => {
                asset_id = chain.as_chain().as_asset_id();
                transaction_type = TransactionType::StakeUndelegate;
                value = message.amount?.amount.clone();
                from_address = message.delegator_address;
                to_address = message.validator_address;
            }
            Message::MsgBeginRedelegate(message) => {
                asset_id = chain.as_chain().as_asset_id();
                transaction_type = TransactionType::StakeRedelegate;
                value = message.amount?.amount.clone();
                from_address = message.delegator_address;
                to_address = message.validator_dst_address;
            }
            Message::MsgWithdrawDelegatorReward(message) => {
                asset_id = chain.as_chain().as_asset_id();
                value = transaction.get_rewards_value(&default_denom)?.to_string();
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
    use primitives::Chain;

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
        let result: TransactionResponse = serde_json::from_str(include_str!("../../testdata/transfer.json")).unwrap();
        let transaction = map_transactions(CosmosChain::Cosmos, vec![result]).first().unwrap().clone();

        assert_eq!(
            transaction,
            Transaction::new(
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
            )
        );
    }

    #[test]
    fn test_transfer_thorchain() {
        let result: TransactionResponse = serde_json::from_str(include_str!("../../testdata/transfer_thorchain.json")).unwrap();
        let transaction = map_transactions(CosmosChain::Thorchain, vec![result]).first().unwrap().clone();

        assert_eq!(
            transaction,
            Transaction::new(
                "C4ED43321E89497C96B7084BE2AA2640EFB10A93A82F396B9FC7A8308F9662AE".to_string(),
                Chain::Thorchain.as_asset_id(),
                "thor1rr6rahhd4sy76a7rdxkjaen2q4k4pw2g06w7qp".to_string(),
                "thor1tpr8cqs2uncwfsggevmha4q4tc9eelu9r00cxx".to_string(),
                None,
                TransactionType::Transfer,
                TransactionState::Confirmed,
                "2000000".to_string(),
                Chain::Thorchain.as_asset_id(),
                "50000000000".to_string(),
                Some("thankyou".to_string()),
                None,
                DateTime::parse_from_rfc3339("2025-10-03T00:39:55Z").unwrap().into(),
            )
        );
    }

    #[test]
    fn test_delegate() {
        let result: TransactionResponse = serde_json::from_str(include_str!("../../testdata/delegate.json")).unwrap();
        let transaction = map_transactions(CosmosChain::Cosmos, vec![result]).first().unwrap().clone();

        assert_eq!(
            transaction,
            Transaction::new(
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
            )
        );
    }

    #[test]
    fn test_rewards() {
        let result: TransactionResponse = serde_json::from_str(include_str!("../../testdata/rewards.json")).unwrap();
        let transaction = map_transactions(CosmosChain::Cosmos, vec![result]).first().unwrap().clone();

        assert_eq!(
            transaction,
            Transaction::new(
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
            )
        );
    }

    #[test]
    fn test_decode_supported_transaction() {
        let payload = "CtQBCo8BChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEm8KLWNvc21vczF6ODM1Y2p4Zno3MzU5NXd1bnF0bjNmbHg2dGdscnN5MDV5NjczdxItY29zbW9zMXo4MzVjanhmejczNTk1d3VucXRuM2ZseDZ0Z2xyc3kwNXk2NzN3Gg8KBXVhdG9tEgYxMDAwMDASQGFlY2IyY2UwZDU1YTg0NTVhNzc2YTMzOWU2ODY1MDE2NmE2YWE0NTVjMDVlZmRkZjQ5ZTAxMWI0MjAzYTI1YTASZwpRCkYKHy9jb3Ntb3MuY3J5cHRvLnNlY3AyNTZrMS5QdWJLZXkSIwohAjhf6Rbk8v7+0NCc4zugr/adpy2yOQikY1pzi6L/SzH2EgQKAggBGOogEhIKDAoFdWF0b20SAzYxOBChxQcaQOl+CCMVx/uR1/yU0RvPKkUADK3LFwo+zsElulf0M34xP00FnS6/51y4FEgn/ewRJokkxy1mPwPvqmK2FBsFVdY=";
        let result = map_transaction_decode(payload.to_string());

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "F09F0730AB6C8C60FBD9252F3844184FF8D463ABE4978937AB7166149F5611FD");
    }

    #[test]
    fn test_decode_unsupported_transaction() {
        let payload = "CooBCocBCiYvc2VpcHJvdG9jb2wuc2VpY2hhaW4ub3JhY2xlLk1zZ0FnZ3JlZ2F0ZUV4Y2hhbmdlUmF0ZVZvdGVdCjQuMjk2ODI3NTE5ODU5MzYzNDIxdWF0b20sMTIwNjgyLjg1MDMyNTUwOTUyMzkwOTIwMnVidGMsNDQ4NS45NDM1MTE1ODEyOTMxMDg2NDZ1ZXRoLDAuMTcxNzU3NzgyMjY4Nzc3Mzg0dW9zbW8sMC4zMDA2MDc3OTA2MDkyMzExNjF1c2VpLDAuOTk5MjM1MTc2MDUxMDgxMDI4dXVzZGMsMS4wMDA0NzA5MTg5NzYyMDM0Njd1dXNkdBIqc2VpMTRxcmN3bXpwZHN6cTBnZWhmcTYzcmFybTdweHF3eG03eGFyY3hqGjFzZWl2YWxvcGVyMTh0cGRldDIya3B2c3d4YXlla3duNTVyeTByNWFjeDRrYWF1dXBrYg==";
        let result = map_transaction_decode(payload.to_string());

        assert!(result.is_none());
    }
}

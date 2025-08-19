use chrono::DateTime;
use primitives::{chain::Chain, Transaction, TransactionState, TransactionType};
use tonlib_core::TonAddress;

use super::model::{HasMemo, Transaction as TonTransaction};

pub struct TonMapper;

impl TonMapper {
    pub fn parse_address(address: &str) -> Option<String> {
        Some(TonAddress::from_hex_str(address).ok()?.to_base64_url())
    }

    pub fn map_transactions(chain: Chain, transactions: Vec<TonTransaction>) -> Vec<Transaction> {
        transactions
            .into_iter()
            .flat_map(|x| Self::map_transaction(chain, x))
            .collect::<Vec<Transaction>>()
    }

    pub fn map_transaction(chain: Chain, transaction: TonTransaction) -> Option<Transaction> {
        if transaction.transaction_type != "TransOrd" {
            return None;
        }

        let asset_id = chain.as_asset_id();
        let state = if transaction.success {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        let created_at = DateTime::from_timestamp(transaction.utime, 0)?;
        let in_msg = transaction.in_msg.as_ref()?;
        let hash = transaction.hash.clone();

        // Handle outgoing transfers (with out messages)
        if transaction.out_msgs.len() == 1 && Self::is_simple_transfer(transaction.out_msgs.first()?) {
            let out_message = transaction.out_msgs.first()?;
            let from = Self::parse_address(&out_message.source.address)?;
            let to = match &out_message.destination {
                Some(destination) => Self::parse_address(&destination.address)?,
                None => return None,
            };
            let value = out_message.value.to_string();
            let memo = Self::extract_memo(out_message);

            return Some(Transaction::new(
                hash,
                asset_id.clone(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                transaction.total_fees.to_string(),
                asset_id,
                value,
                memo,
                None,
                created_at,
            ));
        }

        // Handle incoming transfers (with in message but no out messages)
        if transaction.out_msgs.is_empty() {
            // Check if this is an internal message with value transfer
            if let (Some(msg_type), Some(value), Some(source), Some(destination)) = (&in_msg.msg_type, in_msg.value, &in_msg.source, &in_msg.destination) {
                if msg_type == "int_msg" && value > 0 {
                    let from = Self::parse_address(&source.address)?;
                    let to = Self::parse_address(&destination.address)?;
                    let memo = Self::extract_memo(in_msg);

                    return Some(Transaction::new(
                        hash,
                        asset_id.clone(),
                        from,
                        to,
                        None,
                        TransactionType::Transfer,
                        state,
                        transaction.total_fees.to_string(),
                        asset_id,
                        value.to_string(),
                        memo,
                        None,
                        created_at,
                    ));
                }
            }
        }

        None
    }

    fn is_simple_transfer(out_message: &super::model::OutMessage) -> bool {
        match &out_message.op_code {
            None => true,
            Some(op_code) => op_code == "0x00000000" || op_code == "0x0",
        }
    }

    fn extract_memo<T: HasMemo>(message: &T) -> Option<String> {
        if let Some(comment) = message.comment() {
            if !comment.is_empty() {
                return Some(comment.clone());
            }
        }

        if let Some(decoded_body) = message.decoded_body() {
            if let Some(text) = &decoded_body.text {
                if !text.is_empty() {
                    return Some(text.clone());
                }
            }
            if let Some(comment) = &decoded_body.comment {
                if !comment.is_empty() {
                    return Some(comment.clone());
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::Transaction as TonTransaction;
    use primitives::Chain;
    use serde_json;

    #[test]
    fn test_map_outgoing_transaction_1() {
        let tx: TonTransaction = serde_json::from_str(include_str!("../../testdata/ton_transfer.json")).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, tx);

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.hash, "6fc90afef2393eae1b265d3d4f11eb59ce45b4adcb8e40eb0de00ee27551334f");
        assert_eq!(transaction.asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.from, "EQDKO8nCsVk1DXbYCc2ex4fKGYh3AF2zGeA3mTdqSK2b4BXX");
        assert_eq!(transaction.to, "EQBplgVetLUdBO1zPvCDh8trVzR-RPOGoK1sQ0tXylNfV-Yn");
        assert_eq!(transaction.value, "27013390900");
        assert_eq!(transaction.fee, "1910058");
        assert_eq!(transaction.fee_asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.transaction_type, primitives::TransactionType::Transfer);
        assert_eq!(transaction.state, primitives::TransactionState::Confirmed);
    }

    #[test]
    fn test_map_outgoing_transaction_2() {
        let tx: TonTransaction = serde_json::from_str(include_str!("../../testdata/ton_transfer2.json")).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, tx);

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.hash, "447067aff9c723bde449aafd83de46f6518c61f853196a66fc1bb58cf0fc17f0");
        assert_eq!(transaction.asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.from, "EQDMcfIzYJeFfZz1GLC5DXEduhVUqwe6hxnQQbqZA7wKxA7F");
        assert_eq!(transaction.to, "EQBplgVetLUdBO1zPvCDh8trVzR-RPOGoK1sQ0tXylNfV-Yn");
        assert_eq!(transaction.value, "2030406907");
        assert_eq!(transaction.fee, "5003160");
        assert_eq!(transaction.fee_asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.transaction_type, primitives::TransactionType::Transfer);
        assert_eq!(transaction.state, primitives::TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_with_memo() {
        let tx: TonTransaction = serde_json::from_str(include_str!("../../testdata/tx_with_memo.json")).unwrap();
        let result = TonMapper::map_transaction(Chain::Ton, tx);

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.hash, "420e601674a29c06e7d474e7811526e7aff2bc9cc647cc1fe7754efca5b1f679");
        assert_eq!(transaction.asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.value, "1000000000");
        assert_eq!(transaction.fee, "3674058");
        assert_eq!(transaction.memo, Some("hello-123".to_string()));
        assert_eq!(transaction.transaction_type, primitives::TransactionType::Transfer);
        assert_eq!(transaction.state, primitives::TransactionState::Confirmed);
    }
}

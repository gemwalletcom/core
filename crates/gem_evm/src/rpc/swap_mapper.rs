use alloy_primitives::{hex, Address};
use alloy_sol_types::SolCall;
use chrono::{DateTime, Utc};
use num_bigint::BigUint;

use crate::{
    ethereum_address_checksum,
    rpc::model::{Transaction, TransactionReciept},
    uniswap::{
        actions::{decode_action_data, V4Action},
        command::{V3SwapExactIn, V3_SWAP_EXACT_IN_COMMAND, V4_SWAP_COMMAND},
        contracts::v3::IUniversalRouter,
        deployment::{v3::get_uniswap_router_deployment_by_chain as get_v3_deployment, v4::get_uniswap_deployment_by_chain as get_v4_deployment},
        path::decode_path,
    },
};
use primitives::{AssetId, Chain, TransactionDirection, TransactionState, TransactionSwapMetadata, TransactionType};

// Transfer (index_topic_1 address from, index_topic_2 address to, uint256 value)
const TRANSFER_TOPIC: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

pub struct SwapMapper;

impl SwapMapper {
    pub fn map_uniswap_transaction(
        chain: &Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        let to = transaction.to.clone()?;
        let input_bytes = hex::decode(transaction.input.clone()).ok()?;
        if get_v3_deployment(chain).is_none_or(|dep| to.to_lowercase() != dep.universal_router.to_lowercase())
            && get_v4_deployment(chain).is_none_or(|dep| to.to_lowercase() != dep.universal_router.to_lowercase())
        {
            return None;
        }

        let provider = Self::determine_provider(chain, &to);

        let swap_metadata: Option<TransactionSwapMetadata> =
            if let Some(metadata) = Self::try_map_v3_transaction(chain, provider.clone(), &transaction.from, &input_bytes, transaction_reciept) {
                Some(metadata)
            } else {
                Self::try_map_v4_transaction(chain, provider.clone(), &transaction.from, &input_bytes, transaction_reciept)
            };
        if let Some(swap_metadata) = swap_metadata {
            let from_checksum = ethereum_address_checksum(&transaction.from).ok()?;
            let to_checksum = ethereum_address_checksum(&to).ok()?;
            return Some(primitives::Transaction {
                id: transaction.hash.clone(),
                hash: transaction.hash.clone(),
                asset_id: swap_metadata.from_asset.clone(),
                from: from_checksum.clone(),
                to: from_checksum.clone(),
                contract: Some(to_checksum.clone()),
                transaction_type: TransactionType::Swap,
                state: TransactionState::Confirmed,
                block_number: transaction.block_number.to_string(),
                sequence: transaction.nonce.to_string(),
                fee: transaction_reciept.get_fee().to_string(),
                fee_asset_id: AssetId::from_chain(*chain), // Native asset
                value: transaction.value.to_string(),
                memo: None,
                direction: TransactionDirection::SelfTransfer,
                utxo_inputs: vec![],
                utxo_outputs: vec![],
                metadata: serde_json::to_value(swap_metadata).ok(),
                created_at,
            });
        }
        None
    }

    fn value_from_receipt(to: &str, token: &str, reciept: &TransactionReciept) -> Option<String> {
        for log in reciept.logs.iter() {
            if log.address.to_lowercase() == token.to_lowercase() {
                if log.topics.len() != 3 {
                    continue;
                }
                if log.topics[0].to_lowercase() != TRANSFER_TOPIC {
                    continue;
                }
                let address_bytes = hex::decode(&log.topics[2]).ok()?;
                let topic_2 = Address::from_slice(&address_bytes[address_bytes.len() - 20..]);
                if topic_2.to_checksum(None).to_lowercase() != to.to_lowercase() {
                    continue;
                }

                let value_bytes = hex::decode(&log.data).ok()?;
                let value = BigUint::from_bytes_be(&value_bytes);
                return Some(value.to_string());
            }
        }
        None
    }

    fn determine_provider(_chain: &Chain, _contract: &str) -> Option<String> {
        // FIXME
        None
    }

    fn try_map_v3_transaction(
        chain: &Chain,
        provider: Option<String>,
        from: &str,
        input_bytes: &[u8],
        _transaction_reciept: &TransactionReciept,
    ) -> Option<TransactionSwapMetadata> {
        let execute_call = IUniversalRouter::executeCall::abi_decode(input_bytes).ok()?;
        let commands_vec = execute_call.commands;
        let inputs_vec = execute_call.inputs;
        for (command, input) in commands_vec.iter().zip(inputs_vec.iter()) {
            if command == &V3_SWAP_EXACT_IN_COMMAND {
                let swap_exact_in = V3SwapExactIn::abi_decode(input).ok()?;
                let token_pair = decode_path(&swap_exact_in.path)?;
                let from_token = token_pair.token_in.to_checksum(None);
                let to_token = token_pair.token_out.to_checksum(None);
                return Some(TransactionSwapMetadata {
                    from_value: swap_exact_in.amount_in.to_string(),
                    from_asset: AssetId {
                        chain: *chain,
                        token_id: Some(from_token.clone()),
                    },
                    to_value: Self::value_from_receipt(from, &to_token, _transaction_reciept).unwrap_or(swap_exact_in.amount_out_min.to_string()),
                    to_asset: AssetId {
                        chain: *chain,
                        token_id: Some(to_token),
                    },
                    provider,
                });
            }
        }
        None
    }

    fn try_map_v4_transaction(
        chain: &Chain,
        provider: Option<String>,
        from: &str,
        input_bytes: &[u8],
        _transaction_reciept: &TransactionReciept,
    ) -> Option<TransactionSwapMetadata> {
        let execute_call = IUniversalRouter::executeCall::abi_decode(input_bytes).ok()?;
        let commands_vec = execute_call.commands;
        let inputs_vec = execute_call.inputs;
        for (command, input) in commands_vec.iter().zip(inputs_vec.iter()) {
            if command == &V4_SWAP_COMMAND {
                let v4_swap = decode_action_data(input);
                for action in v4_swap {
                    match action {
                        V4Action::SWAP_EXACT_IN(params) => {
                            let path_keys = params.path;
                            let from_token = path_keys[0].intermediateCurrency.to_checksum(None);
                            let to_token = path_keys[path_keys.len() - 1].intermediateCurrency.to_checksum(None);
                            return Some(TransactionSwapMetadata {
                                from_value: params.amountIn.to_string(),
                                from_asset: AssetId {
                                    chain: *chain,
                                    token_id: Some(from_token.clone()),
                                },
                                to_value: Self::value_from_receipt(from, &to_token, _transaction_reciept)
                                    .unwrap_or_else(|| params.amountOutMinimum.to_string()),
                                to_asset: AssetId {
                                    chain: *chain,
                                    token_id: Some(to_token.clone()),
                                },
                                provider,
                            });
                        }
                        _ => continue,
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::swap_mapper::SwapMapper;
    use primitives::Chain;

    #[test]
    fn test_map_v3_swap_exact_in() {
        // https://etherscan.io/tx/0xfdbc3270b7edf1e63c0aaec9466a71348a1e63bdf069af2d51e9902f996e9d75
        let tx_json = include_str!("test/v3_eth_token_tx.json");
        let tx_value: serde_json::Value = serde_json::from_str(tx_json).unwrap();
        let tx: Transaction = serde_json::from_value(tx_value.get("result").unwrap().clone()).unwrap();

        let receipt_json = include_str!("test/v3_eth_token_tx_receipt.json");
        let receipt_value: serde_json::Value = serde_json::from_str(receipt_json).unwrap();
        let receipt: TransactionReciept = serde_json::from_value(receipt_value.get("result").unwrap().clone()).unwrap();

        let swap_tx = SwapMapper::map_uniswap_transaction(&Chain::Ethereum, &tx, &receipt, Utc::now()).expect("swap_metadata");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x10E11c7368552D5Ab9ef5eED496f614fBAAe9F0D");
        assert_eq!(swap_tx.to, "0x10E11c7368552D5Ab9ef5eED496f614fBAAe9F0D");
        assert_eq!(swap_tx.contract.unwrap(), "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD");
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(swap_tx.value, "18000000000000000");

        assert_eq!(metadata.from_value, "17910000000000000");
        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string()),
            }
        );
        assert_eq!(metadata.to_value, "512854887193301");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some("0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E".to_string()),
            }
        );
    }

    #[test]
    fn test_map_v4_swap_exact_in() {}
}

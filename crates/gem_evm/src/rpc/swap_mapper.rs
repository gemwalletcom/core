use alloy_primitives::{hex, Address};
use alloy_sol_types::SolCall;
use chrono::{DateTime, Utc};
use num_bigint::BigUint;

use crate::{
    ethereum_address_checksum,
    rpc::model::{Transaction, TransactionReciept},
    uniswap::{
        actions::{decode_action_data, V4Action},
        command::{Sweep, UnwrapWeth, V3SwapExactIn, SWEEP_COMMAND, UNWRAP_WETH_COMMAND, V3_SWAP_EXACT_IN_COMMAND, V4_SWAP_COMMAND, WRAP_ETH_COMMAND},
        contracts::v3::IUniversalRouter,
        deployment::get_provider_by_chain_contract,
        path::decode_path,
    },
};
use primitives::{AssetId, Chain, TransactionDirection, TransactionState, TransactionSwapMetadata, TransactionType};

// Transfer (index_topic_1 address from, index_topic_2 address to, uint256 value)
const TRANSFER_TOPIC: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
// Withdrawal (index_topic_1 address src, uint256 wad)
const WITHDRAWAL_TOPIC: &str = "0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65";

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

        let provider = get_provider_by_chain_contract(chain, &to)?;

        let swap_metadata: Option<TransactionSwapMetadata> =
            if let Some(metadata) = Self::try_map_v3_transaction(chain, &provider, &transaction.from, &input_bytes, transaction_reciept) {
                Some(metadata)
            } else {
                Self::try_map_v4_transaction(chain, &provider, &transaction.from, &input_bytes, transaction_reciept)
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

    fn withdraw_value_from_receipt(token: &str, reciept: &TransactionReciept) -> Option<String> {
        for log in reciept.logs.iter() {
            if log.address.to_lowercase() == token.to_lowercase() {
                if log.topics.len() != 2 {
                    continue;
                }
                if log.topics[0].to_lowercase() != WITHDRAWAL_TOPIC {
                    continue;
                }
                let value_bytes = hex::decode(&log.data).ok()?;
                let value = BigUint::from_bytes_be(&value_bytes);
                return Some(value.to_string());
            }
        }
        None
    }

    fn transfer_value_from_receipt(to: &str, token: &str, reciept: &TransactionReciept) -> Option<String> {
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

    fn try_map_v3_transaction(
        chain: &Chain,
        provider: &str,
        from: &str,
        input_bytes: &[u8],
        transaction_reciept: &TransactionReciept,
    ) -> Option<TransactionSwapMetadata> {
        let execute_call = IUniversalRouter::executeCall::abi_decode(input_bytes).ok()?;
        let commands_vec = execute_call.commands;
        let inputs_vec = execute_call.inputs;

        let mut from_asset = None;
        let mut to_asset = None;
        let mut from_value = "".to_string();
        let mut to_value = "".to_string();

        let mut has_wrap = false;
        let mut has_unwrap = false;
        let mut has_sweep = false;
        let mut unwrap_value = "".to_string();
        let mut sweep_value = "".to_string();

        // Check for wrap and unwrap commands first
        for (command, input) in commands_vec.iter().zip(inputs_vec.iter()) {
            if command == &WRAP_ETH_COMMAND {
                has_wrap = true;
            }
            if command == &UNWRAP_WETH_COMMAND {
                let unwrap_weth = UnwrapWeth::abi_decode(input).ok()?;
                has_unwrap = true;
                unwrap_value = unwrap_weth.amount_min.to_string();
            }
            if command == &SWEEP_COMMAND {
                let sweep = Sweep::abi_decode(input).ok()?;
                has_sweep = true;
                sweep_value = sweep.amount_min.to_string();
            }
        }

        // Check V3SwapExactIn
        for (command, input) in commands_vec.iter().zip(inputs_vec.iter()) {
            if command == &V3_SWAP_EXACT_IN_COMMAND {
                let swap_exact_in = V3SwapExactIn::abi_decode(input).ok()?;
                let token_pair = decode_path(&swap_exact_in.path)?;
                let from_token = token_pair.token_in.to_checksum(None);
                let to_token = token_pair.token_out.to_checksum(None);

                from_asset = Some(AssetId {
                    chain: *chain,
                    token_id: if has_wrap { None } else { Some(from_token.clone()) },
                });
                to_asset = Some(AssetId {
                    chain: *chain,
                    token_id: if has_unwrap { None } else { Some(to_token.clone()) },
                });
                from_value = swap_exact_in.amount_in.to_string();
                to_value = if has_unwrap {
                    Self::withdraw_value_from_receipt(&to_token, transaction_reciept).unwrap_or(unwrap_value.clone())
                } else if has_sweep {
                    Self::transfer_value_from_receipt(from, &to_token, transaction_reciept).unwrap_or(sweep_value.clone())
                } else {
                    Self::transfer_value_from_receipt(from, &to_token, transaction_reciept).unwrap_or(swap_exact_in.amount_out_min.to_string())
                }
            }
        }

        if let Some(from_asset) = from_asset {
            if let Some(to_asset) = to_asset {
                if !from_value.is_empty() && !to_value.is_empty() {
                    return Some(TransactionSwapMetadata {
                        from_asset,
                        to_asset,
                        from_value,
                        to_value,
                        provider: Some(provider.to_string()),
                    });
                }
            }
        }
        None
    }

    fn try_map_v4_transaction(
        chain: &Chain,
        provider: &str,
        from: &str,
        input_bytes: &[u8],
        _transaction_reciept: &TransactionReciept,
    ) -> Option<TransactionSwapMetadata> {
        let execute_call = IUniversalRouter::executeCall::abi_decode(input_bytes).ok()?;
        let commands_vec = execute_call.commands;
        let inputs_vec = execute_call.inputs;
        for (command, input) in commands_vec.iter().zip(inputs_vec.iter()) {
            if command == &V4_SWAP_COMMAND {
                if let Ok(decoded_actions_vec) = decode_action_data(input) {
                    for action in decoded_actions_vec {
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
                                    to_value: Self::transfer_value_from_receipt(from, &to_token, _transaction_reciept)
                                        .unwrap_or_else(|| params.amountOutMinimum.to_string()),
                                    to_asset: AssetId {
                                        chain: *chain,
                                        token_id: Some(to_token.clone()),
                                    },
                                    provider: Some(provider.to_string()),
                                });
                            }
                            _ => continue,
                        }
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
    fn test_map_v3_swap_eth_token() {
        // https://app.blocksec.com/explorer/tx/eth/0xfdbc3270b7edf1e63c0aaec9466a71348a1e63bdf069af2d51e9902f996e9d75
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

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: None,
            }
        );
        assert_eq!(metadata.from_value, "17910000000000000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some("0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E".to_string()),
            }
        );
        assert_eq!(metadata.to_value, "512854887193301");
    }

    #[test]
    fn test_map_v3_swap_token_eth() {
        // https://app.blocksec.com/explorer/tx/base/0xc6c2898ddc2d2165bc6c018ec6ebf58d99922c74b9a0e323b50c029d10b09858
        let tx_json = include_str!("test/v3_token_eth_tx.json");
        let tx_value: serde_json::Value = serde_json::from_str(tx_json).unwrap();
        let tx: Transaction = serde_json::from_value(tx_value.get("result").unwrap().clone()).unwrap();

        let receipt_json = include_str!("test/v3_token_eth_tx_receipt.json");
        let receipt_value: serde_json::Value = serde_json::from_str(receipt_json).unwrap();
        let receipt: TransactionReciept = serde_json::from_value(receipt_value.get("result").unwrap().clone()).unwrap();

        let swap_tx = SwapMapper::map_uniswap_transaction(&Chain::Base, &tx, &receipt, Utc::now()).unwrap();
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x985Cf24b63a98510298997Af83a31D8625C09bA5");
        assert_eq!(swap_tx.to, "0x985Cf24b63a98510298997Af83a31D8625C09bA5");
        assert_eq!(swap_tx.contract.unwrap(), "0xFE6508f0015C778Bdcc1fB5465bA5ebE224C9912");
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Base));
        assert_eq!(swap_tx.value, "0");

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some("0x532f27101965dd16442E59d40670FaF5eBB142E4".to_string()),
            }
        );
        assert_eq!(metadata.from_value, "1352497738700000000000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Base,
                token_id: None,
            }
        );
        assert_eq!(metadata.to_value, "29020434785385862");
    }

    #[test]
    fn test_map_v3_swap_pol_usdt() {
        // https://app.blocksec.com/explorer/tx/polygon/0x815759e89e4290873109e482f1f3284cdaca3eb76ff24591a9ac2c6056a2dbcc
        let tx_json = include_str!("test/v3_pol_usdt_tx.json");
        let tx_value: serde_json::Value = serde_json::from_str(tx_json).unwrap();
        let tx: Transaction = serde_json::from_value(tx_value.get("result").unwrap().clone()).unwrap();

        let receipt_json = include_str!("test/v3_pol_usdt_tx_receipt.json");
        let receipt_value: serde_json::Value = serde_json::from_str(receipt_json).unwrap();
        let receipt: TransactionReciept = serde_json::from_value(receipt_value.get("result").unwrap().clone()).unwrap();

        let swap_tx = SwapMapper::map_uniswap_transaction(&Chain::Polygon, &tx, &receipt, Utc::now()).expect("swap_metadata");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x8f4b6cbF3373e065aEb3FEc6027Ff8Ca9a665DE2");
        assert_eq!(swap_tx.to, "0x8f4b6cbF3373e065aEb3FEc6027Ff8Ca9a665DE2");
        assert_eq!(swap_tx.contract.unwrap(), "0xec7BE89e9d109e7e3Fec59c222CF297125FEFda2");
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Polygon));
        assert_eq!(swap_tx.value, "372000000000000000000");

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Polygon,
                token_id: None,
            }
        );
        assert_eq!(metadata.from_value, "372000000000000000000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Polygon,
                token_id: Some("0xc2132D05D31c914a87C6611C10748AEb04B58e8F".to_string()),
            }
        );
        assert_eq!(metadata.to_value, "78290151");
    }

    #[test]
    fn test_map_v3_swap_usdc_paxg() {
        // https://app.blocksec.com/explorer/tx/eth/0x65b5ff389386caf23a9998318d936e434c5bbca850877f1ca03eb246b3ad82e1
        let tx_json = include_str!("test/v3_usdc_paxg_tx.json");
        let tx_value: serde_json::Value = serde_json::from_str(tx_json).unwrap();
        let tx: Transaction = serde_json::from_value(tx_value.get("result").unwrap().clone()).unwrap();

        let receipt_json = include_str!("test/v3_usdc_paxg_receipt.json");
        let receipt_value: serde_json::Value = serde_json::from_str(receipt_json).unwrap();
        let receipt: TransactionReciept = serde_json::from_value(receipt_value.get("result").unwrap().clone()).unwrap();

        let swap_tx = SwapMapper::map_uniswap_transaction(&Chain::Ethereum, &tx, &receipt, Utc::now()).expect("swap_metadata");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0xBa38FE5b73eA5b93d0733CF9eb10aDea6E1E3a2a");
        assert_eq!(swap_tx.to, "0xBa38FE5b73eA5b93d0733CF9eb10aDea6E1E3a2a");
        assert_eq!(swap_tx.contract.unwrap(), "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD");
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(swap_tx.value, "0");

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()),
            }
        );
        assert_eq!(metadata.from_value, "29850000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some("0x45804880De22913dAFE09f4980848ECE6EcbAf78".to_string()),
            }
        );
        assert_eq!(metadata.to_value, "9017156750431593");
    }

    #[test]
    fn test_map_v4_swap_exact_in() {}
}

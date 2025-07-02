use alloy_primitives::{hex, Address};
use alloy_sol_types::SolCall;
use chrono::{DateTime, Utc};
use num_bigint::BigUint;

use crate::{
    ethereum_address_checksum,
    rpc::balance_differ::BalanceDiffer,
    rpc::model::{Transaction, TransactionReciept, TransactionReplayTrace},
    uniswap::{
        actions::{decode_action_data, V4Action},
        command::{Sweep, UnwrapWeth, V3SwapExactIn, SWEEP_COMMAND, UNWRAP_WETH_COMMAND, V3_SWAP_EXACT_IN_COMMAND, V4_SWAP_COMMAND, WRAP_ETH_COMMAND},
        contracts::IUniversalRouter,
        deployment::get_provider_by_chain_contract,
        path::decode_path,
    },
};
use chain_primitives::SwapMapper as BalanceSwapMapper;
use primitives::{AssetId, Chain, TransactionState, TransactionSwapMetadata, TransactionType};

// Transfer (index_topic_1 address from, index_topic_2 address to, uint256 value)
const TRANSFER_TOPIC: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
// Withdrawal (index_topic_1 address src, uint256 wad)
const WITHDRAWAL_TOPIC: &str = "0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65";

pub struct SwapMapper;

impl SwapMapper {
    pub fn map_transaction(
        chain: &Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        trace: Option<&TransactionReplayTrace>,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        // Check if it is a uniswap transaction
        if let Some(provider) = get_provider_by_chain_contract(chain, &transaction.to.clone().unwrap_or_default()) {
            let input_bytes = hex::decode(transaction.input.clone()).ok()?;
            if let Some(swap_metadata) = Self::try_map_transaction(chain, &provider, &transaction.from, &input_bytes, transaction_reciept) {
                return Self::make_swap_transaction(chain, transaction, transaction_reciept, &swap_metadata, created_at);
            }
        }

        // Calculate balance diffs for swap detection
        if let Some(trace) = trace {
            let from = ethereum_address_checksum(&transaction.from).ok()?;
            let differ = BalanceDiffer::new(*chain);
            let diff_map = differ.calculate(trace, transaction_reciept);

            if let Some(diff) = diff_map.get(&from) {
                let native_asset_id = chain.as_asset_id();
                let fee = transaction_reciept.get_fee();
                if let Some(swap_metadata) = BalanceSwapMapper::map_swap(diff, &fee, &native_asset_id, None) {
                    return Self::make_swap_transaction(chain, transaction, transaction_reciept, &swap_metadata, created_at);
                }
            }
        }

        None
    }

    fn make_swap_transaction(
        chain: &Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        metadata: &TransactionSwapMetadata,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        let from_checksum = ethereum_address_checksum(&transaction.from).ok()?;
        let contract_checksum = transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok());
        Some(primitives::Transaction::new(
            transaction.hash.clone(),
            metadata.from_asset.clone(),
            from_checksum.clone(),
            from_checksum.clone(),
            contract_checksum,
            TransactionType::Swap,
            TransactionState::Confirmed,
            transaction_reciept.get_fee().to_string(),
            AssetId::from_chain(*chain),
            transaction.value.to_string(),
            None,
            serde_json::to_value(metadata).ok(),
            created_at,
        ))
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

    pub fn try_map_transaction(
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

        for (command, input) in commands_vec.iter().zip(inputs_vec.iter()) {
            // Check V3SwapExactIn
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
            if command == &V4_SWAP_COMMAND {
                if let Ok(decoded_actions_vec) = decode_action_data(input) {
                    for action in decoded_actions_vec {
                        match action {
                            V4Action::SWAP_EXACT_IN(params) => {
                                let path_keys = params.path;
                                let from_token = params.currencyIn;
                                let to_token = path_keys[path_keys.len() - 1].intermediateCurrency;
                                from_asset = Some(AssetId {
                                    chain: *chain,
                                    token_id: if from_token == Address::ZERO {
                                        None
                                    } else {
                                        Some(from_token.to_checksum(None))
                                    },
                                });
                                to_asset = Some(AssetId {
                                    chain: *chain,
                                    token_id: if to_token == Address::ZERO { None } else { Some(to_token.to_checksum(None)) },
                                });
                                from_value = params.amountIn.to_string();
                                to_value = if to_token == Address::ZERO {
                                    // No logs for native transfer, so we use sweep min value here
                                    sweep_value.clone()
                                } else {
                                    Self::transfer_value_from_receipt(from, &to_token.to_checksum(None), transaction_reciept)?
                                };
                            }
                            _ => continue,
                        }
                    }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::swap_mapper::SwapMapper;
    use primitives::{Chain, JsonRpcResult};

    #[test]
    fn test_map_v4_swap_eth_dai() {
        let tx_json = include_str!("../../tests/data/v4_eth_dai_tx.json");
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(tx_json).unwrap().result;

        let receipt_json = include_str!("../../tests/data/v4_eth_dai_tx_receipt.json");
        let receipt_value: JsonRpcResult<TransactionReciept> = serde_json::from_str(receipt_json).unwrap();
        let receipt = receipt_value.result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Unichain, &transaction, &receipt, None, DateTime::default()).expect("swap_metadata");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.to, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.contract.unwrap(), "0xEf740bf23aCaE26f6492B10de645D6B98dC8Eaf3");
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Unichain));
        assert_eq!(swap_tx.value, "1000000000000000");

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: None,
            }
        );
        assert_eq!(metadata.from_value, "995000000000000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: Some("0x20CAb320A855b39F724131C69424240519573f81".to_string()),
            }
        );
        assert_eq!(metadata.to_value, "2696771430516915192");
    }

    #[test]
    fn test_map_v4_swap_usdc_eth() {
        let tx_json = include_str!("../../tests/data/v4_usdc_eth_tx.json");
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(tx_json).unwrap().result;

        let receipt_json = include_str!("../../tests/data/v4_usdc_eth_tx_receipt.json");
        let receipt_value: JsonRpcResult<TransactionReciept> = serde_json::from_str(receipt_json).unwrap();
        let receipt = receipt_value.result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Unichain, &transaction, &receipt, None, DateTime::default()).expect("swap_metadata");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.to, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.contract.unwrap(), "0xEf740bf23aCaE26f6492B10de645D6B98dC8Eaf3");
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Unichain));
        assert_eq!(swap_tx.value, "0");

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: Some("0x078D782b760474a361dDA0AF3839290b0EF57AD6".to_string()),
            }
        );
        assert_eq!(metadata.from_value, "2132953");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: None,
            }
        );
        assert_eq!(metadata.to_value, "1155057703771482");
    }

    #[test]
    fn test_map_v3_swap_eth_token() {
        // https://app.blocksec.com/explorer/tx/eth/0xfdbc3270b7edf1e63c0aaec9466a71348a1e63bdf069af2d51e9902f996e9d75
        let tx_json = include_str!("../../tests/data/v3_eth_token_tx.json");
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(tx_json).unwrap().result;

        let receipt_json = include_str!("../../tests/data/v3_eth_token_tx_receipt.json");
        let receipt_value: JsonRpcResult<TransactionReciept> = serde_json::from_str(receipt_json).unwrap();
        let receipt = receipt_value.result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Ethereum, &transaction, &receipt, None, DateTime::default()).expect("swap_metadata");
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
        let tx_json = include_str!("../../tests/data/v3_token_eth_tx.json");
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(tx_json).unwrap().result;

        let receipt_json = include_str!("../../tests/data/v3_token_eth_tx_receipt.json");
        let receipt_value: JsonRpcResult<TransactionReciept> = serde_json::from_str(receipt_json).unwrap();
        let receipt = receipt_value.result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Base, &transaction, &receipt, None, DateTime::default()).unwrap();
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
        let tx_json = include_str!("../../tests/data/v3_pol_usdt_tx.json");
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(tx_json).unwrap().result;

        let receipt_json = include_str!("../../tests/data/v3_pol_usdt_tx_receipt.json");
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(receipt_json).unwrap().result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Polygon, &transaction, &receipt, None, DateTime::default()).expect("swap_metadata");
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
        let tx_json = include_str!("../../tests/data/v3_usdc_paxg_tx.json");
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(tx_json).unwrap().result;

        let receipt_json = include_str!("../../tests/data/v3_usdc_paxg_receipt.json");
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(receipt_json).unwrap().result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Ethereum, &transaction, &receipt, None, DateTime::default()).unwrap();

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
    fn test_swap_from_balance_diff() {
        let tx_json = include_str!("../../tests/data/trace_replay_tx.json");
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(tx_json).unwrap().result;

        let receipt_json = include_str!("../../tests/data/trace_replay_tx_receipt.json");
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(receipt_json).unwrap().result;

        let trace_json = include_str!("../../tests/data/trace_replay_tx_trace.json");
        let trace = serde_json::from_str::<JsonRpcResult<TransactionReplayTrace>>(trace_json).unwrap().result;

        let swap_tx = SwapMapper::map_transaction(
            &Chain::Ethereum,
            &transaction,
            &receipt,
            Some(&trace),
            DateTime::from_timestamp(1735671600, 0).expect("invalid timestamp"),
        )
        .unwrap();

        assert_eq!(swap_tx.from, "0x52A07c930157d07D9EffD147ecF41C5cBbC6000c");
        assert_eq!(swap_tx.to, "0x52A07c930157d07D9EffD147ecF41C5cBbC6000c");
        assert_eq!(swap_tx.contract.unwrap(), "0x111111125421cA6dc452d289314280a0f8842A65");
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(swap_tx.value, "0");

        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some("0xD0eC028a3D21533Fdd200838F39c85B03679285D".to_string()),
            }
        );
        assert_eq!(metadata.from_value, "780000000000000000000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: None,
            }
        );
        assert_eq!(metadata.to_value, "158035947652936307");
    }

    #[test]
    fn test_map_transaction_v2_token_eth() {
        let tx_json = include_str!("../../tests/data/v2_token_eth_tx.json");
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(tx_json).unwrap().result;

        let receipt_json = include_str!("../../tests/data/v2_token_eth_tx_receipt.json");
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(receipt_json).unwrap().result;

        let trace_json = include_str!("../../tests/data/v2_token_eth_tx_trace.json");
        let trace = serde_json::from_str::<JsonRpcResult<TransactionReplayTrace>>(trace_json).unwrap().result;

        let swap_tx = SwapMapper::map_transaction(
            &Chain::Ethereum,
            &transaction,
            &receipt,
            Some(&trace),
            DateTime::from_timestamp(1735671600, 0).expect("invalid timestamp"),
        )
        .unwrap();

        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap());

        assert!(metadata.is_ok());
    }
}

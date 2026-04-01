use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use chrono::{DateTime, Utc};
use std::str::FromStr;

use super::{
    mapper::TRANSFER_TOPIC,
    parsers::{make_swap_transaction, try_map_balance_diff_swap},
};
use crate::{
    address::ethereum_address_from_topic,
    ethereum_address_checksum,
    registry::ContractRegistry,
    rpc::{
        model::{Transaction, TransactionReciept, TransactionReplayTrace},
        staking_mapper::ethereum_value_from_log_data,
    },
    uniswap::{
        actions::{V4Action, decode_action_data},
        command::{SWEEP_COMMAND, Sweep, UNWRAP_WETH_COMMAND, UnwrapWeth, V3_SWAP_EXACT_IN_COMMAND, V3SwapExactIn, V4_SWAP_COMMAND, WRAP_ETH_COMMAND},
        contracts::IUniversalRouter,
        deployment::get_provider_by_chain_contract,
        path::decode_path,
    },
};
use primitives::{AssetId, Chain, TransactionSwapMetadata, decode_hex};

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
        contract_registry: Option<&ContractRegistry>,
    ) -> Option<primitives::Transaction> {
        if let Some(to_address) = &transaction.to
            && let Some(provider) = get_provider_by_chain_contract(chain, to_address)
        {
            let input_bytes = decode_hex(&transaction.input).ok()?;
            if let Some(swap_metadata) = Self::try_map_uniswap_transaction(chain, &provider, &transaction.from, &input_bytes, transaction_reciept) {
                return make_swap_transaction(chain, transaction, transaction_reciept, &swap_metadata, created_at);
            }
        }

        // Calculate balance diffs for swap detection
        if let Some(trace) = trace {
            let contract_registry = contract_registry?;
            let to = Address::from_str(&transaction.to.clone().unwrap_or_default()).ok()?;
            let registry_entry = contract_registry.get_by_address(&to, *chain)?;
            if let Some(swap_metadata) = try_map_balance_diff_swap(chain, &transaction.from, Some(trace), transaction_reciept, Some(registry_entry.provider.to_string())) {
                return make_swap_transaction(chain, transaction, transaction_reciept, &swap_metadata, created_at);
            }
        }

        None
    }

    fn withdraw_value_from_receipt(token: &str, reciept: &TransactionReciept) -> Option<String> {
        let token = ethereum_address_checksum(token).ok()?;

        reciept.logs.iter().find_map(|log| {
            (ethereum_address_checksum(&log.address).ok()? == token && log.topics.len() == 2 && log.topics.first().is_some_and(|topic| topic == WITHDRAWAL_TOPIC))
                .then(|| ethereum_value_from_log_data(&log.data, 0, 64))
                .flatten()
                .map(|value| value.to_string())
        })
    }

    fn transfer_value_from_receipt(to: &str, token: &str, reciept: &TransactionReciept) -> Option<String> {
        let to = ethereum_address_checksum(to).ok()?;
        let token = ethereum_address_checksum(token).ok()?;

        reciept.logs.iter().find_map(|log| {
            (ethereum_address_checksum(&log.address).ok()? == token
                && log.topics.len() == 3
                && log.topics.first().is_some_and(|topic| topic == TRANSFER_TOPIC)
                && ethereum_address_from_topic(log.topics.get(2)?)? == to)
                .then(|| ethereum_value_from_log_data(&log.data, 0, 64))
                .flatten()
                .map(|value| value.to_string())
        })
    }

    pub fn try_map_uniswap_transaction(chain: &Chain, provider: &str, from: &str, input_bytes: &[u8], transaction_reciept: &TransactionReciept) -> Option<TransactionSwapMetadata> {
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
            if command == &V4_SWAP_COMMAND
                && let Ok(decoded_actions_vec) = decode_action_data(input)
            {
                for action in decoded_actions_vec {
                    match action {
                        V4Action::SWAP_EXACT_IN(params) => {
                            let path_keys = params.path;
                            let from_token = params.currencyIn;
                            let to_token = if path_keys.is_empty() {
                                continue;
                            } else {
                                path_keys[path_keys.len() - 1].intermediateCurrency
                            };
                            from_asset = Some(AssetId {
                                chain: *chain,
                                token_id: if from_token == Address::ZERO { None } else { Some(from_token.to_checksum(None)) },
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

        if let Some(from_asset) = from_asset
            && let Some(to_asset) = to_asset
            && !from_value.is_empty()
            && !to_value.is_empty()
        {
            return Some(TransactionSwapMetadata {
                from_asset,
                to_asset,
                from_value,
                to_value,
                provider: Some(provider.to_string()),
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::testkit::TOKEN_USDC_ADDRESS;
    use primitives::{
        Chain, JsonRpcResult, TransactionType,
        asset_constants::{POLYGON_USDT_TOKEN_ID, UNICHAIN_DAI_TOKEN_ID, UNICHAIN_USDC_TOKEN_ID},
        contract_constants::{ETHEREUM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT, UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT},
    };

    #[test]
    fn test_map_v4_swap_eth_dai() {
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(include_str!("../../testdata/v4_eth_dai_tx.json"))
            .unwrap()
            .result;
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(include_str!("../../testdata/v4_eth_dai_tx_receipt.json"))
            .unwrap()
            .result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Unichain, &transaction, &receipt, None, DateTime::default(), None).expect("swap_metadata");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.to, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.contract.unwrap(), UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT);
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Unichain));
        assert_eq!(swap_tx.value, "1000000000000000");

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: None
            }
        );
        assert_eq!(metadata.from_value, "995000000000000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: Some(UNICHAIN_DAI_TOKEN_ID.to_string()),
            }
        );
        assert_eq!(metadata.to_value, "2696771430516915192");
    }

    #[test]
    fn test_map_v4_swap_usdc_eth() {
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(include_str!("../../testdata/v4_usdc_eth_tx.json"))
            .unwrap()
            .result;
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(include_str!("../../testdata/v4_usdc_eth_tx_receipt.json"))
            .unwrap()
            .result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Unichain, &transaction, &receipt, None, DateTime::default(), None).expect("swap_metadata");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.to, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.contract.unwrap(), UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT);
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Unichain));
        assert_eq!(swap_tx.value, "0");

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: Some(UNICHAIN_USDC_TOKEN_ID.to_string()),
            }
        );
        assert_eq!(metadata.from_value, "2132953");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: None
            }
        );
        assert_eq!(metadata.to_value, "1155057703771482");
    }

    #[test]
    fn test_map_v3_swap_eth_token() {
        // https://app.blocksec.com/explorer/tx/eth/0xfdbc3270b7edf1e63c0aaec9466a71348a1e63bdf069af2d51e9902f996e9d75
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(include_str!("../../testdata/v3_eth_token_tx.json"))
            .unwrap()
            .result;
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(include_str!("../../testdata/v3_eth_token_tx_receipt.json"))
            .unwrap()
            .result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Ethereum, &transaction, &receipt, None, DateTime::default(), None).expect("swap_metadata");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x10E11c7368552D5Ab9ef5eED496f614fBAAe9F0D");
        assert_eq!(swap_tx.to, "0x10E11c7368552D5Ab9ef5eED496f614fBAAe9F0D");
        assert_eq!(swap_tx.contract.unwrap(), ETHEREUM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT);
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(swap_tx.value, "18000000000000000");

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: None
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
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(include_str!("../../testdata/v3_token_eth_tx.json"))
            .unwrap()
            .result;
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(include_str!("../../testdata/v3_token_eth_tx_receipt.json"))
            .unwrap()
            .result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Base, &transaction, &receipt, None, DateTime::default(), None).expect("swap_metadata");
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
                token_id: None
            }
        );
        assert_eq!(metadata.to_value, "29020434785385862");
    }

    #[test]
    fn test_map_v3_swap_pol_usdt() {
        // https://app.blocksec.com/explorer/tx/polygon/0x815759e89e4290873109e482f1f3284cdaca3eb76ff24591a9ac2c6056a2dbcc
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(include_str!("../../testdata/v3_pol_usdt_tx.json"))
            .unwrap()
            .result;
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(include_str!("../../testdata/v3_pol_usdt_tx_receipt.json"))
            .unwrap()
            .result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Polygon, &transaction, &receipt, None, DateTime::default(), None).expect("swap_metadata");
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
                token_id: None
            }
        );
        assert_eq!(metadata.from_value, "372000000000000000000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Polygon,
                token_id: Some(POLYGON_USDT_TOKEN_ID.to_string()),
            }
        );
        assert_eq!(metadata.to_value, "78290151");
    }

    #[test]
    fn test_map_v3_swap_usdc_paxg() {
        // https://app.blocksec.com/explorer/tx/eth/0x65b5ff389386caf23a9998318d936e434c5bbca850877f1ca03eb246b3ad82e1
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(include_str!("../../testdata/v3_usdc_paxg_tx.json"))
            .unwrap()
            .result;
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(include_str!("../../testdata/v3_usdc_paxg_receipt.json"))
            .unwrap()
            .result;

        let swap_tx = SwapMapper::map_transaction(&Chain::Ethereum, &transaction, &receipt, None, DateTime::default(), None).expect("swap_metadata");
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0xBa38FE5b73eA5b93d0733CF9eb10aDea6E1E3a2a");
        assert_eq!(swap_tx.to, "0xBa38FE5b73eA5b93d0733CF9eb10aDea6E1E3a2a");
        assert_eq!(swap_tx.contract.unwrap(), ETHEREUM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT);
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(swap_tx.value, "0");

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some(TOKEN_USDC_ADDRESS.to_string()),
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
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(include_str!("../../testdata/trace_replay_tx.json"))
            .unwrap()
            .result;
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(include_str!("../../testdata/trace_replay_tx_receipt.json"))
            .unwrap()
            .result;
        let trace = serde_json::from_str::<JsonRpcResult<TransactionReplayTrace>>(include_str!("../../testdata/trace_replay_tx_trace.json"))
            .unwrap()
            .result;

        let contract_registry = ContractRegistry::default();
        let swap_tx = SwapMapper::map_transaction(
            &Chain::Ethereum,
            &transaction,
            &receipt,
            Some(&trace),
            DateTime::from_timestamp(1735671600, 0).expect("invalid timestamp"),
            Some(&contract_registry),
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
                token_id: None
            }
        );
        assert_eq!(metadata.to_value, "158035947652936307");
    }

    #[test]
    fn test_map_transaction_v2_token_eth() {
        let transaction = serde_json::from_str::<JsonRpcResult<Transaction>>(include_str!("../../testdata/v2_token_eth_tx.json"))
            .unwrap()
            .result;
        let receipt = serde_json::from_str::<JsonRpcResult<TransactionReciept>>(include_str!("../../testdata/v2_token_eth_tx_receipt.json"))
            .unwrap()
            .result;
        let trace = serde_json::from_str::<JsonRpcResult<TransactionReplayTrace>>(include_str!("../../testdata/v2_token_eth_tx_trace.json"))
            .unwrap()
            .result;

        let contract_registry = ContractRegistry::default();
        let swap_tx = SwapMapper::map_transaction(
            &Chain::Ethereum,
            &transaction,
            &receipt,
            Some(&trace),
            DateTime::from_timestamp(1735671600, 0).expect("invalid timestamp"),
            Some(&contract_registry),
        )
        .unwrap();

        assert!(swap_tx.metadata.is_some());
    }

    #[test]
    fn test_v4_swap_empty_path_no_panic() {
        use crate::uniswap::{actions::V4Action, contracts::v4::IV4Router};
        use alloy_primitives::Address;

        let action = V4Action::SWAP_EXACT_IN(IV4Router::ExactInputParams {
            currencyIn: Address::ZERO,
            path: vec![],
            amountIn: 1000000000000000000_u128,
            amountOutMinimum: 0,
        });

        let encoded_actions = crate::uniswap::actions::encode_actions(&[action]);
        let decoded_actions = crate::uniswap::actions::decode_action_data(&encoded_actions);
        assert!(decoded_actions.is_ok());

        let actions = decoded_actions.unwrap();
        assert_eq!(actions.len(), 1);

        if let V4Action::SWAP_EXACT_IN(params) = &actions[0] {
            assert!(params.path.is_empty());
        }
    }
}

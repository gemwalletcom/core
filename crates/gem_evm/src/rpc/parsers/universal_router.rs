use alloy_primitives::Address;
use alloy_sol_types::SolCall;

use crate::{
    address::ethereum_address_from_topic,
    ethereum_address_checksum,
    rpc::{mapper::TRANSFER_TOPIC, model::TransactionReciept, staking_mapper::ethereum_value_from_log_data},
    uniswap::{
        actions::{V4Action, decode_action_data},
        command::{SWEEP_COMMAND, Sweep, UNWRAP_WETH_COMMAND, UnwrapWeth, V3_SWAP_EXACT_IN_COMMAND, V3SwapExactIn, V4_SWAP_COMMAND, WRAP_ETH_COMMAND},
        contracts::IUniversalRouter,
        deployment::get_provider_by_chain_contract,
        path::decode_path,
    },
};
use primitives::{AssetId, Chain, Transaction as PrimitivesTransaction, TransactionSwapMetadata, decode_hex};

use super::{ParseContext, ProtocolParser, make_swap_transaction, try_map_balance_diff_swap};

const WITHDRAWAL_TOPIC: &str = "0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65";

pub struct UniversalRouterParser;

impl ProtocolParser for UniversalRouterParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        context
            .transaction
            .to
            .as_ref()
            .is_some_and(|to| get_provider_by_chain_contract(context.chain, to).is_some())
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let to = context.transaction.to.as_ref()?;
        let provider = get_provider_by_chain_contract(context.chain, to)?;
        let input_bytes = decode_hex(&context.transaction.input).ok()?;

        let metadata = decode_execute_swap(context.chain, &provider, &context.transaction.from, &input_bytes, context.receipt)
            .or_else(|| try_map_balance_diff_swap(context.chain, &context.transaction.from, context.trace, context.receipt, Some(provider.clone())))?;

        make_swap_transaction(context.chain, context.transaction, context.receipt, &metadata, context.created_at)
    }
}

pub fn decode_execute_swap(chain: &Chain, provider: &str, from: &str, input_bytes: &[u8], receipt: &TransactionReciept) -> Option<TransactionSwapMetadata> {
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
                withdraw_value_from_receipt(&to_token, receipt).unwrap_or(unwrap_value.clone())
            } else if has_sweep {
                transfer_value_from_receipt(from, &to_token, receipt).unwrap_or(sweep_value.clone())
            } else {
                transfer_value_from_receipt(from, &to_token, receipt).unwrap_or(swap_exact_in.amount_out_min.to_string())
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
                            sweep_value.clone()
                        } else {
                            transfer_value_from_receipt(from, &to_token.to_checksum(None), receipt)?
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

fn withdraw_value_from_receipt(token: &str, receipt: &TransactionReciept) -> Option<String> {
    let token = ethereum_address_checksum(token).ok()?;

    receipt.logs.iter().find_map(|log| {
        (ethereum_address_checksum(&log.address).ok()? == token && log.topics.len() == 2 && log.topics.first().is_some_and(|topic| topic == WITHDRAWAL_TOPIC))
            .then(|| ethereum_value_from_log_data(&log.data, 0, 64))
            .flatten()
            .map(|value| value.to_string())
    })
}

fn transfer_value_from_receipt(to: &str, token: &str, receipt: &TransactionReciept) -> Option<String> {
    let to = ethereum_address_checksum(to).ok()?;
    let token = ethereum_address_checksum(token).ok()?;

    receipt.logs.iter().find_map(|log| {
        (ethereum_address_checksum(&log.address).ok()? == token
            && log.topics.len() == 3
            && log.topics.first().is_some_and(|topic| topic == TRANSFER_TOPIC)
            && ethereum_address_from_topic(log.topics.get(2)?)? == to)
            .then(|| ethereum_value_from_log_data(&log.data, 0, 64))
            .flatten()
            .map(|value| value.to_string())
    })
}

#[cfg(test)]
mod tests {
    use crate::provider::testkit::TOKEN_USDC_ADDRESS;
    use crate::registry::ContractRegistry;
    use crate::rpc::model::{Transaction, TransactionReciept, TransactionReplayTrace};
    use crate::rpc::parsers::ProtocolParsers;
    use chrono::DateTime;
    use primitives::{
        AssetId, Chain, TransactionSwapMetadata, TransactionType,
        asset_constants::{POLYGON_USDT_TOKEN_ID, UNICHAIN_DAI_TOKEN_ID, UNICHAIN_USDC_TOKEN_ID},
        contract_constants::{ETHEREUM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT, UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT},
        testkit::json_rpc::load_json_rpc_result,
    };

    fn map_swap(chain: &Chain, transaction: &Transaction, receipt: &TransactionReciept) -> primitives::Transaction {
        ProtocolParsers::map_transaction(chain, transaction, receipt, None, None, DateTime::default()).unwrap()
    }

    fn map_swap_with_trace(
        chain: &Chain,
        transaction: &Transaction,
        receipt: &TransactionReciept,
        trace: &TransactionReplayTrace,
        registry: &ContractRegistry,
    ) -> primitives::Transaction {
        ProtocolParsers::map_transaction(chain, transaction, receipt, Some(trace), Some(registry), DateTime::from_timestamp(1735671600, 0).unwrap()).unwrap()
    }

    #[test]
    fn test_map_v4_swap_eth_dai() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v4_eth_dai_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/v4_eth_dai_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Unichain, &transaction, &receipt);
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
                token_id: Some(UNICHAIN_DAI_TOKEN_ID.to_string())
            }
        );
        assert_eq!(metadata.to_value, "2696771430516915192");
    }

    #[test]
    fn test_map_v4_swap_usdc_eth() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v4_usdc_eth_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/v4_usdc_eth_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Unichain, &transaction, &receipt);
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
                token_id: Some(UNICHAIN_USDC_TOKEN_ID.to_string())
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
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v3_eth_token_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/v3_eth_token_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Ethereum, &transaction, &receipt);
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
                token_id: Some("0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E".to_string())
            }
        );
        assert_eq!(metadata.to_value, "512854887193301");
    }

    #[test]
    fn test_map_v3_swap_token_eth() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v3_token_eth_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/v3_token_eth_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Base, &transaction, &receipt);
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
                token_id: Some("0x532f27101965dd16442E59d40670FaF5eBB142E4".to_string())
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
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v3_pol_usdt_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/v3_pol_usdt_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Polygon, &transaction, &receipt);
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
                token_id: Some(POLYGON_USDT_TOKEN_ID.to_string())
            }
        );
        assert_eq!(metadata.to_value, "78290151");
    }

    #[test]
    fn test_map_v3_swap_usdc_paxg() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v3_usdc_paxg_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/v3_usdc_paxg_receipt.json"));

        let swap_tx = map_swap(&Chain::Ethereum, &transaction, &receipt);
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
                token_id: Some(TOKEN_USDC_ADDRESS.to_string())
            }
        );
        assert_eq!(metadata.from_value, "29850000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some("0x45804880De22913dAFE09f4980848ECE6EcbAf78".to_string())
            }
        );
        assert_eq!(metadata.to_value, "9017156750431593");
    }

    #[test]
    fn test_swap_from_balance_diff() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/trace_replay_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/trace_replay_tx_receipt.json"));
        let trace = load_json_rpc_result::<TransactionReplayTrace>(include_str!("../../../testdata/trace_replay_tx_trace.json"));

        let contract_registry = ContractRegistry::default();
        let swap_tx = map_swap_with_trace(&Chain::Ethereum, &transaction, &receipt, &trace, &contract_registry);

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
                token_id: Some("0xD0eC028a3D21533Fdd200838F39c85B03679285D".to_string())
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
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v2_token_eth_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/v2_token_eth_tx_receipt.json"));
        let trace = load_json_rpc_result::<TransactionReplayTrace>(include_str!("../../../testdata/v2_token_eth_tx_trace.json"));

        let contract_registry = ContractRegistry::default();
        let swap_tx = map_swap_with_trace(&Chain::Ethereum, &transaction, &receipt, &trace, &contract_registry);
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

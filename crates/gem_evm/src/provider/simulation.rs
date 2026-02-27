use alloy_primitives::{Address, hex};
use async_trait::async_trait;
use chain_primitives::{BalanceDiff, BalanceDiffMap};
use chain_traits::ChainSimulation;
use gem_client::Client;
use num_bigint::BigInt;
use num_traits::Num;
use primitives::{AssetId, Chain, SimulationInput, SimulationResult};
use std::collections::{HashMap, HashSet};

use crate::ethereum_address_checksum;
use crate::jsonrpc::TransactionObject;
use crate::rpc::client::EthereumClient;
use crate::rpc::debug_trace::{CallFrame, CallLog, PrestateDiffResult};
use crate::rpc::mapper::TRANSFER_TOPIC;

#[async_trait]
impl<C: Client + Clone> ChainSimulation for EthereumClient<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn std::error::Error + Send + Sync>> {
        let tx: TransactionObject = serde_json::from_str(&input.encoded_transaction)?;

        let (prestate, call_frame) = futures::try_join!(self.debug_trace_call_prestate(&tx), self.debug_trace_call_logs(&tx),)?;

        let success = call_frame.error.is_none();
        let error = call_frame.error.clone().or_else(|| call_frame.revert_reason.clone());
        let gas_used = call_frame.gas_used.as_deref().and_then(|g| u64::from_str_radix(g.trim_start_matches("0x"), 16).ok());

        let balance_changes = map_balance_changes(self.get_chain(), &prestate, &call_frame);

        Ok(SimulationResult {
            success,
            error,
            logs: vec![],
            units_consumed: gas_used,
            balance_changes,
        })
    }
}

fn map_balance_changes(chain: Chain, prestate: &PrestateDiffResult, call_frame: &CallFrame) -> BalanceDiffMap {
    let mut map: BalanceDiffMap = HashMap::new();

    // Native balance diffs from prestateTracer
    let all_addresses: HashSet<&String> = prestate.pre.keys().chain(prestate.post.keys()).collect();

    for address in all_addresses {
        let pre_balance = prestate.pre.get(address).and_then(|s| s.balance.as_deref()).and_then(parse_hex_bigint);
        let post_balance = prestate.post.get(address).and_then(|s| s.balance.as_deref()).and_then(parse_hex_bigint);

        let (from_value, to_value, diff) = match (pre_balance, post_balance) {
            (Some(pre), Some(post)) => {
                let d = &post - &pre;
                if d == BigInt::from(0) {
                    continue;
                }
                (Some(pre), Some(post), d)
            }
            (Some(pre), None) => {
                let d = -pre.clone();
                (Some(pre), Some(BigInt::from(0)), d)
            }
            (None, Some(post)) => {
                let d = post.clone();
                (Some(BigInt::from(0)), Some(post), d)
            }
            (None, None) => continue,
        };

        let checksum = ethereum_address_checksum(address).unwrap_or_default();
        map.entry(checksum).or_default().push(BalanceDiff {
            asset_id: AssetId { chain, token_id: None },
            from_value: Some(from_value.unwrap_or_default()),
            to_value: Some(to_value.unwrap_or_default()),
            diff,
        });
    }

    // ERC20 token transfers from callTracer logs
    let mut token_transfers: HashMap<String, HashMap<String, BigInt>> = HashMap::new();

    for log in &call_frame.logs {
        if let Some((from, to, value)) = parse_transfer_log(log) {
            let token_address = ethereum_address_checksum(&log.address).unwrap_or_default();
            *token_transfers.entry(from).or_default().entry(token_address.clone()).or_default() -= value.clone();
            *token_transfers.entry(to).or_default().entry(token_address).or_default() += value;
        }
    }

    for (address, tokens) in token_transfers {
        for (token_address, net_diff) in tokens {
            if net_diff != BigInt::from(0) {
                map.entry(address.clone()).or_default().push(BalanceDiff {
                    asset_id: AssetId {
                        chain,
                        token_id: Some(token_address),
                    },
                    from_value: None,
                    to_value: None,
                    diff: net_diff,
                });
            }
        }
    }

    map
}

fn parse_transfer_log(log: &CallLog) -> Option<(String, String, BigInt)> {
    if log.topics.len() < 3 || log.topics[0] != TRANSFER_TOPIC {
        return None;
    }

    let from_bytes = hex::decode(log.topics[1].trim_start_matches("0x")).ok()?;
    let to_bytes = hex::decode(log.topics[2].trim_start_matches("0x")).ok()?;

    if from_bytes.len() != 32 || to_bytes.len() != 32 {
        return None;
    }

    let from = Address::from_slice(&from_bytes[12..]).to_checksum(None);
    let to = Address::from_slice(&to_bytes[12..]).to_checksum(None);

    let data = log.data.trim_start_matches("0x");
    let value = BigInt::from_str_radix(data, 16).ok()?;

    Some((from, to, value))
}

fn parse_hex_bigint(hex_str: &str) -> Option<BigInt> {
    let stripped = hex_str.trim_start_matches("0x");
    if stripped.is_empty() {
        return Some(BigInt::from(0));
    }
    BigInt::from_str_radix(stripped, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::debug_trace::AccountState;

    #[test]
    fn test_native_balance_changes() {
        let mut pre = HashMap::new();
        pre.insert(
            "0x52a07c930157d07d9effd147ecf41c5cbbc6000c".to_string(),
            AccountState {
                balance: Some("0x28268111de83a9d".to_string()),
            },
        );
        let mut post = HashMap::new();
        post.insert(
            "0x52a07c930157d07d9effd147ecf41c5cbbc6000c".to_string(),
            AccountState {
                balance: Some("0x4bd382b322e4810".to_string()),
            },
        );

        let prestate = PrestateDiffResult { pre, post };
        let call_frame = CallFrame {
            gas_used: None,
            output: None,
            error: None,
            revert_reason: None,
            logs: vec![],
        };

        let result = map_balance_changes(Chain::Ethereum, &prestate, &call_frame);

        let sender = "0x52A07c930157d07D9EffD147ecF41C5cBbC6000c";
        let diffs = result.get(sender).expect("sender diffs");
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].asset_id, AssetId::from_chain(Chain::Ethereum));
        assert!(diffs[0].diff > BigInt::from(0));
    }

    #[test]
    fn test_erc20_transfer_from_logs() {
        let prestate = PrestateDiffResult::default();
        let call_frame = CallFrame {
            gas_used: None,
            output: None,
            error: None,
            revert_reason: None,
            logs: vec![CallLog {
                address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                topics: vec![
                    TRANSFER_TOPIC.to_string(),
                    "0x00000000000000000000000052a07c930157d07d9effd147ecf41c5cbbc6000c".to_string(),
                    "0x000000000000000000000000def1c0ded9bec7f1a1670819833240f027b25eff".to_string(),
                ],
                data: "0x000000000000000000000000000000000000000000000000000000003b9aca00".to_string(),
            }],
        };

        let result = map_balance_changes(Chain::Ethereum, &prestate, &call_frame);

        let sender = "0x52A07c930157d07D9EffD147ecF41C5cBbC6000c";
        let sender_diffs = result.get(sender).expect("sender diffs");
        assert_eq!(sender_diffs.len(), 1);
        assert!(sender_diffs[0].diff < BigInt::from(0));
        assert_eq!(sender_diffs[0].asset_id.token_id.as_deref(), Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));

        let receiver = "0xDef1C0ded9bec7F1a1670819833240f027b25EfF";
        let receiver_diffs = result.get(receiver).expect("receiver diffs");
        assert_eq!(receiver_diffs.len(), 1);
        assert!(receiver_diffs[0].diff > BigInt::from(0));
    }

    #[test]
    fn test_no_changes_produces_empty_map() {
        let prestate = PrestateDiffResult::default();
        let call_frame = CallFrame {
            gas_used: None,
            output: None,
            error: None,
            revert_reason: None,
            logs: vec![],
        };

        let result = map_balance_changes(Chain::Ethereum, &prestate, &call_frame);
        assert!(result.is_empty());
    }
}

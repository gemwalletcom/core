use crate::{
    ethereum_address_checksum,
    rpc::{
        mapper::TRANSFER_TOPIC,
        model::{Diff, Log, TransactionReciept, TransactionReplayTrace},
    },
};
use alloy_primitives::{hex, Address};
use chain_primitives::{BalanceDiff, BalanceDiffMap};
use num_bigint::{BigInt, BigUint};
use num_traits::Num;
use primitives::{AssetId, Chain};
use std::collections::HashMap;

struct TransferLog {
    pub from: String,
    pub to: String,
    pub value: BigInt,
}

#[derive(Debug)]
pub struct BalanceDiffer {
    pub chain: Chain,
}

impl BalanceDiffer {
    pub fn new(chain: Chain) -> Self {
        Self { chain }
    }

    pub fn calculate(&self, trace: &TransactionReplayTrace, receipt: &TransactionReciept) -> BalanceDiffMap {
        let mut map: BalanceDiffMap = HashMap::new();

        // Native balance diff
        for (address, state) in &trace.state_diff {
            if let Diff::Change(change) = &state.balance {
                let checksum_address = ethereum_address_checksum(address).unwrap_or_default();
                let from_value = BigInt::from_str_radix(&change.from_to.from[2..], 16).ok();
                let to_value = BigInt::from_str_radix(&change.from_to.to[2..], 16).ok();

                if let (Some(from_bigint), Some(to_bigint)) = (from_value.clone(), to_value.clone()) {
                    let diff_value = to_bigint - from_bigint;
                    let diff = BalanceDiff {
                        asset_id: AssetId {
                            chain: self.chain,
                            token_id: None,
                        },
                        from_value,
                        to_value,
                        diff: diff_value,
                    };
                    map.entry(checksum_address).or_default().push(diff);
                }
            }
        }

        // ERC20 token net transfers - collect all transfers per address/token and calculate net
        let mut token_transfers: HashMap<String, HashMap<String, BigInt>> = HashMap::new();

        for log in &receipt.logs {
            if let Some(transfer) = self.parse_log(log) {
                let token_address = ethereum_address_checksum(&log.address).unwrap_or_default();

                // Subtract from sender
                *token_transfers.entry(transfer.from).or_default().entry(token_address.clone()).or_default() -= transfer.value.clone();

                // Add to receiver
                *token_transfers.entry(transfer.to).or_default().entry(token_address.clone()).or_default() += transfer.value;
            }
        }

        // Convert net transfers to BalanceDiff entries
        for (address, tokens) in token_transfers {
            for (token_address, net_diff) in tokens {
                if net_diff != BigInt::from(0) {
                    let asset_id = AssetId {
                        chain: self.chain,
                        token_id: Some(token_address),
                    };

                    let diff = BalanceDiff {
                        asset_id,
                        from_value: None,
                        to_value: None,
                        diff: net_diff,
                    };

                    map.entry(address.clone()).or_default().push(diff);
                }
            }
        }

        map
    }

    fn parse_log(&self, log: &Log) -> Option<TransferLog> {
        // Transfer(address,address,uint256)
        if log.topics.is_empty() || log.topics[0] != TRANSFER_TOPIC || log.topics.len() < 3 {
            return None;
        }

        // topics[1] is from, topics[2] is to. They are 32 bytes, address is last 20 bytes.
        let from_bytes = hex::decode(&log.topics[1]).ok()?;
        let to_bytes = hex::decode(&log.topics[2]).ok()?;

        if from_bytes.len() != 32 || to_bytes.len() != 32 {
            return None;
        }

        let from = Address::from_slice(&from_bytes[12..]).to_checksum(None);
        let to = Address::from_slice(&to_bytes[12..]).to_checksum(None);

        let value = BigUint::from_str_radix(&log.data[2..], 16).ok()?;

        Some(TransferLog { from, to, value: value.into() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_jsonrpc::types::JsonRpcResponse;
    use primitives::Chain;
    use std::str::FromStr;

    #[test]
    fn test_calculate() {
        // https://etherscan.io/tx/0x23fe2ead060a3812a1f03c2e082b6fc8888b7c655a8f58f4ed19de00e8c9aaa6
        let json_str = include_str!("../../tests/data/trace_replay_tx_trace.json");
        let trace_replay_transaction = serde_json::from_str::<JsonRpcResponse<TransactionReplayTrace>>(json_str).unwrap().result;

        let json_str = include_str!("../../tests/data/trace_replay_tx_receipt.json");
        let receipt = serde_json::from_str::<JsonRpcResponse<TransactionReciept>>(json_str).unwrap().result;

        let differ = BalanceDiffer::new(Chain::Ethereum);
        let diff_map = differ.calculate(&trace_replay_transaction, &receipt);

        let sender_address = "0x52A07c930157d07D9EffD147ecF41C5cBbC6000c";
        let sender_diffs = diff_map.get(sender_address).unwrap();

        // Check native balance change: from 0x28268111de83a9d (180821357773732509) to 0x4bd382b322e4810 (341490904926668816)
        let native_diff = sender_diffs
            .iter()
            .find(|d| d.asset_id == AssetId::from_chain(Chain::Ethereum))
            .expect("Native diff not found in sender's diffs");

        assert_eq!(native_diff.from_value, Some(BigInt::from_str("180821357773732509").unwrap()));
        assert_eq!(native_diff.to_value, Some(BigInt::from_str("341490904926668816").unwrap()));
        assert_eq!(native_diff.diff, BigInt::from_str("160669547152936307").unwrap());

        // Check ERC20 token net change: -780 NEWT
        let newt_asset_id = AssetId {
            chain: Chain::Ethereum,
            token_id: Some("0xD0eC028a3D21533Fdd200838F39c85B03679285D".to_string()),
        };
        let token_diff = sender_diffs
            .iter()
            .find(|d| d.asset_id == newt_asset_id)
            .expect("Token diff not found in sender's diffs");

        assert_eq!(token_diff.from_value, None);
        assert_eq!(token_diff.to_value, None);
        assert_eq!(token_diff.diff, BigInt::from_str("-780000000000000000000").unwrap());

        let pool_address = "0x000000000004444c5dc75cB358380D2e3dE08A90";
        let pool_diffs = diff_map.get(pool_address).unwrap();

        // Check native balance change: from 0x8d849264a8118b46324 to 0x8d846e21f2859c0bab1
        let contract_native_diff = pool_diffs
            .iter()
            .find(|d| d.asset_id == AssetId::from_chain(Chain::Ethereum))
            .expect("Native diff not found in contract's diffs");

        assert_eq!(contract_native_diff.from_value, Some(BigInt::from_str("41768699565210634314532").unwrap()));
        assert_eq!(contract_native_diff.to_value, Some(BigInt::from_str("41768536262063981378225").unwrap()));
        assert_eq!(contract_native_diff.diff, BigInt::from_str("-163303146652936307").unwrap()); // negative diff

        // Check ERC20 token net change: +778.05 NEWT
        let pool_token_diff = pool_diffs
            .iter()
            .find(|d| d.asset_id == newt_asset_id)
            .expect("Token diff not found in contract's diffs");

        assert_eq!(pool_token_diff.from_value, None);
        assert_eq!(pool_token_diff.to_value, None);
        assert_eq!(pool_token_diff.diff, BigInt::from_str("778050000000000000000").unwrap());

        let rabby_address = "0x39041F1B366fE33F9A5a79dE5120F2Aee2577ebc";
        let rabby_diffs = diff_map.get(rabby_address).unwrap();
        let rabby_token_diff = rabby_diffs
            .iter()
            .find(|d| d.asset_id == newt_asset_id)
            .expect("Token diff not found in Rabby's diffs");

        assert_eq!(rabby_token_diff.from_value, None);
        assert_eq!(rabby_token_diff.to_value, None);
        assert_eq!(rabby_token_diff.diff, BigInt::from_str("1950000000000000000").unwrap());
    }
}

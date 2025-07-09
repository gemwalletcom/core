use crate::SUI_COIN_TYPE;
use crate::SUI_COIN_TYPE_FULL;
use chain_primitives::{BalanceDiff, SwapMapper};
use chrono::TimeZone;
use chrono::Utc;
use num_bigint::BigUint;
use primitives::SwapProvider;
use primitives::TransactionSwapMetadata;
use primitives::{chain::Chain, Asset, AssetId, AssetType, Transaction, TransactionState, TransactionType};

use super::model::BalanceChange;

use super::{
    constants::{SUI_STAKE_EVENT, SUI_UNSTAKE_EVENT},
    model::{CoinMetadata, Digest as SuiTransaction, EventStake, EventUnstake, GasUsed},
};

pub struct SuiMapper;

impl SuiMapper {
    const CHAIN: Chain = Chain::Sui;

    pub fn get_fee(gas_used: GasUsed) -> BigUint {
        let computation_cost = gas_used.computation_cost;
        let storage_cost = gas_used.storage_cost;
        let storage_rebate = gas_used.storage_rebate;

        let cost = computation_cost.clone() + storage_cost.clone();
        if storage_rebate >= cost {
            return BigUint::from(0u32);
        }
        computation_cost + storage_cost - storage_rebate
    }

    pub fn map_transaction(transaction: SuiTransaction) -> Option<Transaction> {
        let chain = Self::CHAIN;
        let balance_changes = transaction.balance_changes.unwrap_or_default();
        let effects = transaction.effects.clone();
        let hash = transaction.digest.clone();
        let fee = Self::get_fee(effects.gas_used.clone());
        let created_at = Utc.timestamp_millis_opt(transaction.timestamp_ms as i64).unwrap();
        let state = if effects.status.status == "success" {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        let owner = effects.gas_object.owner.get_address_owner();
        let sui_coin_type = chain.as_denom()?;

        // system & token transfer
        if transaction.events.is_empty() && (balance_changes.len() == 2 || balance_changes.len() == 3) {
            let (from_change, to_change) =
                if balance_changes.len() == 2 && balance_changes[0].coin_type == sui_coin_type && balance_changes[1].coin_type == sui_coin_type {
                    if balance_changes[0].amount < balance_changes[1].amount {
                        (balance_changes[0].clone(), balance_changes[1].clone())
                    } else {
                        (balance_changes[1].clone(), balance_changes[0].clone())
                    }
                } else if balance_changes.len() == 3 && balance_changes[0].coin_type == sui_coin_type {
                    if balance_changes[1].amount < balance_changes[2].amount {
                        (balance_changes[1].clone(), balance_changes[2].clone())
                    } else {
                        (balance_changes[2].clone(), balance_changes[1].clone())
                    }
                } else {
                    return None;
                };

            let asset_id = if from_change.coin_type == sui_coin_type {
                chain.as_asset_id()
            } else {
                AssetId::from_token(chain, &from_change.coin_type)
            };
            let from = from_change.owner.get_address_owner()?;
            let to = to_change.owner.get_address_owner()?;
            let value = to_change.amount;

            let transaction = Transaction::new(
                hash,
                asset_id,
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                fee.to_string(),
                chain.as_asset_id(),
                value.to_string(),
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        // stake
        if transaction.events.len() == 1 && transaction.events.first().is_some_and(|e| e.event_type == SUI_STAKE_EVENT) {
            let event = transaction.events.first()?;
            let event_json = event.parsed_json.clone()?;
            let stake = serde_json::from_value::<EventStake>(event_json).ok()?;

            let transaction = Transaction::new(
                hash,
                chain.as_asset_id(),
                stake.staker_address,
                stake.validator_address,
                None,
                TransactionType::StakeDelegate,
                state,
                fee.to_string(),
                chain.as_asset_id(),
                stake.amount,
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        // swap
        if transaction.events.iter().any(|x| x.event_type.contains("Swap")) {
            let owner_balance_changes = balance_changes
                .iter()
                .filter(|x| x.owner.get_address_owner() == owner)
                .cloned()
                .collect::<Vec<_>>();
            // TODO: Handle other swap providers
            let swap = match owner_balance_changes.len() {
                2 => Self::map_swap_from_balance_changes(owner_balance_changes.clone(), &fee)?,
                3 => {
                    let owner_balance_changes_filtered = owner_balance_changes
                        .iter()
                        .filter(|x| x.coin_type != SUI_COIN_TYPE)
                        .cloned()
                        .collect::<Vec<_>>();
                    Self::map_swap_from_balance_changes(owner_balance_changes_filtered.clone(), &fee)?
                }
                _ => return None,
            };

            let transaction = Transaction::new(
                hash,
                chain.as_asset_id(),
                owner.clone()?,
                owner.clone()?,
                None,
                TransactionType::Swap,
                TransactionState::Confirmed,
                fee.to_string(),
                chain.as_asset_id(),
                swap.clone().from_value,
                None,
                serde_json::to_value(swap.clone()).ok(),
                created_at,
            );
            return Some(transaction);
        }

        // unstake
        if transaction.events.len() == 1 && transaction.events.first().is_some_and(|e| e.event_type == SUI_UNSTAKE_EVENT) {
            let event = transaction.events.first()?;
            let event_json = event.parsed_json.clone()?;
            let stake: EventUnstake = serde_json::from_value::<EventUnstake>(event_json).ok()?;
            let value = stake.principal_amount; // add reward amount

            let transaction = Transaction::new(
                hash,
                chain.as_asset_id(),
                stake.staker_address,
                stake.validator_address,
                None,
                TransactionType::StakeUndelegate,
                state,
                fee.to_string(),
                chain.as_asset_id(),
                value,
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        None
    }

    pub fn map_swap_from_balance_changes(balance_changes: Vec<BalanceChange>, fee: &BigUint) -> Option<TransactionSwapMetadata> {
        let balance_diffs: Vec<BalanceDiff> = balance_changes
            .into_iter()
            .map(|change| BalanceDiff {
                asset_id: Self::map_asset_id(&change.coin_type),
                from_value: None,
                to_value: None,
                diff: change.amount,
            })
            .collect();

        let native_asset_id = Chain::Sui.as_asset_id();
        SwapMapper::map_swap(&balance_diffs, fee, &native_asset_id, Some(SwapProvider::Cetus.id().to_owned()))
    }

    pub fn map_asset_id(coin_type: &str) -> AssetId {
        match coin_type {
            SUI_COIN_TYPE | SUI_COIN_TYPE_FULL => Chain::Sui.as_asset_id(),
            _ => AssetId::from_token(Chain::Sui, coin_type),
        }
    }

    pub fn map_token(chain: Chain, metadata: CoinMetadata) -> Asset {
        Asset::new(
            AssetId::from_token(chain, &metadata.id.clone()),
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            AssetType::TOKEN,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::JsonRpcResult;

    #[test]
    fn test_transaction_swap_token_to_token() {
        let file_content = include_str!("../../testdata/swap_token_to_token.json");
        let result: JsonRpcResult<SuiTransaction> = serde_json::from_str(file_content).unwrap();

        let transaction = SuiMapper::map_transaction(result.result).unwrap();
        let expected = TransactionSwapMetadata {
            from_asset: AssetId::from_token(Chain::Sui, "0x5c60d2434f7487703dffecb958b99827f4e1e3eef4cbbf1091318cb0b0a787c2::coin::COIN"),
            from_value: "5489450364172".to_string(),
            to_asset: AssetId::from_token(Chain::Sui, "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC"),
            to_value: "103045380".to_string(),
            provider: Some(SwapProvider::Cetus.id().to_owned()),
        };

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_swap_sui_to_token() {
        let file_content = include_str!("../../testdata/swap_sui_to_token.json");
        let result: JsonRpcResult<SuiTransaction> = serde_json::from_str(file_content).unwrap();

        let transaction = SuiMapper::map_transaction(result.result).unwrap();
        let expected = TransactionSwapMetadata {
            from_asset: Chain::Sui.as_asset_id(),
            from_value: "1000000000".to_string(),
            to_asset: AssetId::from_token(Chain::Sui, "0x356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59::wal::WAL"),
            to_value: "6634805122".to_string(),
            provider: Some(SwapProvider::Cetus.id().to_owned()),
        };

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_sui_transfer() {
        let file_content = include_str!("../../testdata/transfer_sui.json");
        let result: JsonRpcResult<SuiTransaction> = serde_json::from_str(file_content).unwrap();

        let transaction = SuiMapper::map_transaction(result.result).unwrap();

        assert_eq!(transaction.hash, "CJ16PEqq49KFp758iEVwxEkd3CwP7zDfqGYLuLuu9Z63");
        assert_eq!(transaction.from, "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6");
        assert_eq!(transaction.to, "0x9d6b98b18fd26b5efeec68d020dcf1be7a94c2c315353779bc6b3aed44188ddf");
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, "100000000");
        assert_eq!(transaction.asset_id, Chain::Sui.as_asset_id());
    }

    #[test]
    fn test_transaction_token_transfer() {
        let file_content = include_str!("../../testdata/transfer_token.json");
        let result: JsonRpcResult<SuiTransaction> = serde_json::from_str(file_content).unwrap();

        let transaction = SuiMapper::map_transaction(result.result).unwrap();

        assert_eq!(transaction.hash, "4MjZc5GrgP6peCwbA1LyS5rXNdJxKjWbd2TrSDZQBXKf");
        assert_eq!(transaction.from, "0x5987852fa4832e820031fd373d6ec8a2cfe84031a59851b8aca60c79b1aa9c2d");
        assert_eq!(transaction.to, "0xf0e295b8e40b4d554f1f0e4ac78d79227f1921ccd3059d017d0225553384c781");
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, "10231646");
        assert_eq!(
            transaction.asset_id,
            AssetId::from_token(Chain::Sui, "0xc060006111016b8a020ad5b33834984a437aaa7d3c74c18e09a95d48aceab08c::coin::COIN")
        );
    }
}

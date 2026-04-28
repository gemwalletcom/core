use std::collections::{HashMap, HashSet};

use num_bigint::BigInt;
use primitives::{AssetId, Chain, SimulationBalanceChange, SimulationResult, SimulationSeverity, SimulationWarning, SimulationWarningType};
use serde_json::Value;

use crate::models::{SimulateTransactionValue, TokenBalance};

pub fn map_simulation_result(account_keys: &[String], signer_addresses: &HashSet<String>, simulation: SimulateTransactionValue) -> SimulationResult {
    if let Some(err) = simulation.err {
        return SimulationResult::new(vec![simulation_error_warning(err)], vec![]);
    }

    SimulationResult {
        balance_changes: map_balance_changes(
            account_keys,
            signer_addresses,
            &simulation.pre_balances,
            &simulation.post_balances,
            &simulation.pre_token_balances.unwrap_or_default(),
            &simulation.post_token_balances.unwrap_or_default(),
        ),
        ..Default::default()
    }
}

fn simulation_error_warning(error: Value) -> SimulationWarning {
    let message = match error {
        Value::String(message) => message,
        error => error.to_string(),
    };
    SimulationWarning::new(SimulationSeverity::Critical, SimulationWarningType::ValidationError, Some(message))
}

fn map_balance_changes(
    account_keys: &[String],
    signer_addresses: &HashSet<String>,
    pre_balances: &[u64],
    post_balances: &[u64],
    pre_token_balances: &[TokenBalance],
    post_token_balances: &[TokenBalance],
) -> Vec<SimulationBalanceChange> {
    let mut pre_asset_values = get_asset_values(pre_token_balances, signer_addresses);
    let mut post_asset_values = get_asset_values(post_token_balances, signer_addresses);

    add_native_asset_values(&mut pre_asset_values, account_keys, signer_addresses, pre_balances);
    add_native_asset_values(&mut post_asset_values, account_keys, signer_addresses, post_balances);

    map_asset_value_changes(pre_asset_values, post_asset_values)
}

fn add_native_asset_values(asset_values: &mut HashMap<AssetId, BigInt>, account_keys: &[String], signer_addresses: &HashSet<String>, balances: &[u64]) {
    for (index, address) in account_keys.iter().enumerate() {
        if !signer_addresses.contains(address) {
            continue;
        }

        let Some(balance) = balances.get(index).copied() else {
            continue;
        };
        add_asset_value(asset_values, AssetId::from_chain(Chain::Solana), BigInt::from(balance));
    }
}

fn map_asset_value_changes(pre_asset_values: HashMap<AssetId, BigInt>, post_asset_values: HashMap<AssetId, BigInt>) -> Vec<SimulationBalanceChange> {
    let asset_ids: HashSet<_> = pre_asset_values.keys().chain(post_asset_values.keys()).cloned().collect();
    let mut balance_changes: Vec<_> = asset_ids
        .into_iter()
        .filter_map(|asset_id| {
            let pre_value = pre_asset_values.get(&asset_id).cloned().unwrap_or_default();
            let post_value = post_asset_values.get(&asset_id).cloned().unwrap_or_default();
            let value = post_value - pre_value;

            if value == BigInt::from(0) {
                None
            } else {
                Some(SimulationBalanceChange {
                    asset_id,
                    value: value.to_string(),
                })
            }
        })
        .collect();
    balance_changes.sort_by_key(|change| change.asset_id.to_string());
    balance_changes
}

fn add_asset_value(asset_values: &mut HashMap<AssetId, BigInt>, asset_id: AssetId, value: BigInt) {
    if value != BigInt::from(0) {
        *asset_values.entry(asset_id).or_default() += value;
    }
}

fn get_asset_values(token_balances: &[TokenBalance], signer_addresses: &HashSet<String>) -> HashMap<AssetId, BigInt> {
    let mut amounts = HashMap::new();

    for token_balance in token_balances {
        if !signer_addresses.contains(&token_balance.owner) {
            continue;
        }

        let asset_id = AssetId::from_token(Chain::Solana, &token_balance.mint);
        add_asset_value(&mut amounts, asset_id, BigInt::from(token_balance.get_amount()));
    }

    amounts
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::asset_constants::{SOLANA_USDC_ASSET_ID, SOLANA_USDC_TOKEN_ID};

    use crate::models::TokenAmount;

    fn token_balance(account_index: i64, mint: &str, owner: &str, amount: u64) -> TokenBalance {
        TokenBalance::new(account_index, mint.to_string(), owner.to_string(), TokenAmount { amount: BigUint::from(amount) })
    }

    fn signers(addresses: &[&str]) -> HashSet<String> {
        addresses.iter().map(|address| address.to_string()).collect()
    }

    #[test]
    fn test_map_balance_changes() {
        let account_keys = vec!["wallet".to_string(), "recipient".to_string()];
        let pre_balances = vec![1_000_000_000, 0];
        let post_balances = vec![899_995_000, 100_000_000];
        let pre_tokens = vec![token_balance(2, SOLANA_USDC_TOKEN_ID, "wallet", 1_000_000)];
        let post_tokens = vec![token_balance(2, SOLANA_USDC_TOKEN_ID, "wallet", 250_000)];

        let changes = map_balance_changes(&account_keys, &signers(&["wallet"]), &pre_balances, &post_balances, &pre_tokens, &post_tokens);

        assert_eq!(
            changes,
            vec![
                SimulationBalanceChange {
                    asset_id: AssetId::from_chain(Chain::Solana),
                    value: "-100005000".to_string(),
                },
                SimulationBalanceChange {
                    asset_id: SOLANA_USDC_ASSET_ID.clone(),
                    value: "-750000".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_map_balance_changes_ignores_non_signers() {
        let account_keys = vec!["wallet".to_string(), "recipient".to_string()];
        let pre_balances = vec![1_000_000_000, 0];
        let post_balances = vec![999_995_000, 5_000_000];
        let pre_tokens = vec![token_balance(2, SOLANA_USDC_TOKEN_ID, "recipient", 1_000_000)];
        let post_tokens = vec![token_balance(2, SOLANA_USDC_TOKEN_ID, "recipient", 2_000_000)];

        let changes = map_balance_changes(&account_keys, &signers(&["wallet"]), &pre_balances, &post_balances, &pre_tokens, &post_tokens);

        assert_eq!(
            changes,
            vec![SimulationBalanceChange {
                asset_id: AssetId::from_chain(Chain::Solana),
                value: "-5000".to_string(),
            }]
        );
    }

    #[test]
    fn test_map_simulation_result_returns_validation_warning_for_failed_simulation() {
        let result = map_simulation_result(
            &[],
            &HashSet::new(),
            SimulateTransactionValue {
                err: Some(serde_json::json!({"InstructionError":[1, "InvalidArgument"]})),
                pre_balances: vec![],
                post_balances: vec![],
                pre_token_balances: None,
                post_token_balances: None,
            },
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Critical,
                SimulationWarningType::ValidationError,
                Some("{\"InstructionError\":[1,\"InvalidArgument\"]}".to_string()),
            )]
        );
        assert_eq!(result.balance_changes, vec![]);
    }
}

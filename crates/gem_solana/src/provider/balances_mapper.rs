use primitives::{AssetBalance, AssetId, Chain};

use crate::models::balances::SolanaBalance;
use crate::models::{TokenAccountInfo, ValueResult};

pub fn map_coin_balance(balance: &SolanaBalance) -> AssetBalance {
    let asset_id = AssetId::from_chain(Chain::Solana);
    AssetBalance::new(asset_id, balance.value.to_string())
}

pub fn map_token_balances(accounts: &ValueResult<Vec<TokenAccountInfo>>, token_ids: &[String]) -> Vec<AssetBalance> {
    accounts
        .value
        .iter()
        .zip(token_ids.iter())
        .map(|(account, token_id)| {
            let balance_amount = account
                .account
                .data
                .parsed
                .info
                .token_amount
                .as_ref()
                .map(|ta| ta.amount.to_string())
                .unwrap_or_else(|| "0".to_string());
            AssetBalance::new(AssetId::from_token(Chain::Solana, token_id), balance_amount)
        })
        .collect()
}

pub fn map_single_token_balance(account: &TokenAccountInfo, token_id: &str) -> AssetBalance {
    let balance_amount = account
        .account
        .data
        .parsed
        .info
        .token_amount
        .as_ref()
        .map(|ta| ta.amount.to_string())
        .unwrap_or_else(|| "0".to_string());
    AssetBalance::new(AssetId::from_token(Chain::Solana, token_id), balance_amount)
}

pub fn map_token_accounts(accounts: &ValueResult<Vec<TokenAccountInfo>>, token_id: &str) -> Vec<AssetBalance> {
    if let Some(account) = accounts.value.first() {
        vec![map_single_token_balance(account, token_id)]
    } else {
        vec![AssetBalance::new(AssetId::from_token(Chain::Solana, token_id), "0".to_string())]
    }
}

pub fn map_balance_staking(stake_accounts: Vec<TokenAccountInfo>) -> Option<AssetBalance> {
    let total_staked: u64 = stake_accounts.iter().map(|x| x.account.lamports).sum();

    Some(AssetBalance::new_staking(
        AssetId::from_chain(Chain::Solana),
        total_staked.to_string(),
        "0".to_string(),
        "0".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::JsonRpcResult;

    #[test]
    fn test_map_coin_balance() {
        let result: JsonRpcResult<SolanaBalance> = serde_json::from_str(include_str!("../../testdata/balance_coin.json")).unwrap();

        let balance_result = map_coin_balance(&result.result);

        assert_eq!(balance_result.asset_id.chain, Chain::Solana);
        assert_eq!(balance_result.balance.available.to_string(), "1366309311");
    }

    #[test]
    fn test_map_single_token_balance() {
        let result: JsonRpcResult<ValueResult<Vec<TokenAccountInfo>>> = serde_json::from_str(include_str!("../../testdata/balance_spl_token.json")).unwrap();

        let token_account = &result.result.value[0];
        let token_id = "2zMMhcVQEXDtdE6vsFS7S7D5oUodfJHE8vd1gnBouauv";
        let balance_result = map_single_token_balance(token_account, token_id);

        assert_eq!(balance_result.asset_id.chain, Chain::Solana);
        assert_eq!(balance_result.balance.available.to_string(), "75071408");
    }

    #[test]
    fn test_map_token_balances() {
        let result: JsonRpcResult<ValueResult<Vec<TokenAccountInfo>>> = serde_json::from_str(include_str!("../../testdata/balance_spl_token.json")).unwrap();

        let token_ids = vec!["2zMMhcVQEXDtdE6vsFS7S7D5oUodfJHE8vd1gnBouauv".to_string()];
        let balances = map_token_balances(&result.result, &token_ids);

        assert_eq!(balances.len(), 1);
        assert_eq!(balances[0].asset_id.chain, Chain::Solana);
        assert_eq!(balances[0].balance.available.to_string(), "75071408");
    }

    #[test]
    fn test_map_staking_balance() {
        let result: JsonRpcResult<Vec<TokenAccountInfo>> = serde_json::from_str(include_str!("../../testdata/balance_staking.json")).unwrap();
        let staking_balance = map_balance_staking(result.result).unwrap();

        assert_eq!(staking_balance.asset_id.chain, Chain::Solana);
        assert_eq!(staking_balance.balance.available.to_string(), "0");
        assert_eq!(staking_balance.balance.staked.to_string(), "363542610");
    }
}

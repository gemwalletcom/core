use crate::models::staking::SuiStakeDelegation;
use crate::models::Balance as SuiBalance;
use crate::{SUI_COIN_TYPE, SUI_COIN_TYPE_FULL};
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Balance, Chain};

pub fn map_balance_coin(balance: SuiBalance) -> AssetBalance {
    AssetBalance::new_balance(
        Chain::Sui.as_asset_id(),
        Balance::coin_balance(BigUint::try_from(balance.total_balance).unwrap_or_default()),
    )
}

pub fn map_balance_tokens(balances: Vec<SuiBalance>, token_ids: Vec<String>) -> Vec<AssetBalance> {
    token_ids
        .into_iter()
        .map(|token_id| {
            let balance = balances
                .iter()
                .find(|b| coin_type_matches(&b.coin_type, &token_id))
                .map(|b| &b.total_balance)
                .cloned()
                .unwrap_or_default();

            AssetBalance::new_balance(
                AssetId::from_token(Chain::Sui, &token_id),
                Balance::coin_balance(BigUint::try_from(balance).unwrap_or_default()),
            )
        })
        .collect()
}

pub fn map_balance_staking(delegations: Vec<SuiStakeDelegation>) -> Option<AssetBalance> {
    if delegations.is_empty() {
        return None;
    }

    let staked = delegations
        .iter()
        .flat_map(|delegation| &delegation.stakes)
        .map(|stake| &stake.principal + stake.estimated_reward.as_ref().unwrap_or(&num_bigint::BigInt::from(0)))
        .sum::<num_bigint::BigInt>();

    Some(AssetBalance::new_balance(
        Chain::Sui.as_asset_id(),
        Balance::stake_balance(BigUint::from(0u32), BigUint::from(0u32), BigUint::try_from(staked).unwrap_or_default(), BigUint::from(0u32), None),
    ))
}

pub fn map_staking_balance(delegations: Vec<SuiStakeDelegation>) -> AssetBalance {
    let staked_total = delegations
        .iter()
        .flat_map(|delegation| &delegation.stakes)
        .map(|stake| &stake.principal + stake.estimated_reward.as_ref().unwrap_or(&num_bigint::BigInt::from(0)))
        .sum::<num_bigint::BigInt>();

    AssetBalance::new_balance(
        Chain::Sui.as_asset_id(),
        Balance::stake_balance(
            BigUint::from(0u32),
            BigUint::from(0u32),
            BigUint::try_from(staked_total).unwrap_or_default(),
            BigUint::from(0u32),
            None,
        ),
    )
}

pub fn map_assets_balances(balances: Vec<SuiBalance>) -> Vec<AssetBalance> {
    balances
        .into_iter()
        .filter_map(|balance| {
            let asset_id = if balance.coin_type == SUI_COIN_TYPE || balance.coin_type == SUI_COIN_TYPE_FULL {
                None // Skip native coin as it's handled separately
            } else {
                Some(AssetId::from_token(Chain::Sui, &balance.coin_type))
            };

            asset_id.map(|asset_id| AssetBalance::new_balance(asset_id, Balance::coin_balance(BigUint::try_from(balance.total_balance).unwrap_or_default())))
        })
        .collect()
}

fn coin_type_matches(coin_type: &str, token_id: &str) -> bool {
    // Remove 0x prefix and normalize for comparison
    let coin_type_normalized = coin_type.strip_prefix("0x").unwrap_or(coin_type).to_lowercase();
    let token_id_normalized = token_id.strip_prefix("0x").unwrap_or(token_id).to_lowercase();

    coin_type_normalized == token_id_normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_map_coin_balance() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/balance_coin.json")).unwrap();
        let balance: SuiBalance = serde_json::from_value(response["result"].clone()).unwrap();

        let result = map_balance_coin(balance);
        assert_eq!(result.balance.available, BigUint::from(52855428706_u64));
        assert_eq!(result.asset_id.chain, Chain::Sui);
    }

    #[test]
    fn test_map_token_balances() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../testdata/balance_tokens.json")).unwrap();
        let balances: Vec<SuiBalance> = serde_json::from_value(response["result"].clone()).unwrap();

        let token_ids = vec![
            "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC".to_string(),
            "0xda1644f58a955833a15abae24f8cc65b5bd8152ce013fde8be0a6a3dcf51fe36::token::TOKEN".to_string(),
        ];

        let result = map_balance_tokens(balances, token_ids);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].balance.available, BigUint::from(3685298_u64)); // USDC balance
        assert_eq!(result[1].balance.available, BigUint::from(1000_u64)); // TOKEN balance
    }

    #[test]
    fn test_coin_type_matches() {
        assert!(coin_type_matches("0x2::sui::SUI", "0x2::sui::SUI"));
        assert!(coin_type_matches("0x2::sui::SUI", "2::sui::SUI"));
        assert!(coin_type_matches("2::sui::SUI", "0x2::sui::SUI"));
        assert!(!coin_type_matches("0x2::sui::SUI", "0x3::token::TOKEN"));
    }

    #[test]
    fn test_map_balance_staking() {
        use primitives::JsonRpcResult;

        let response: JsonRpcResult<Vec<SuiStakeDelegation>> = serde_json::from_str(include_str!("../../testdata/stakes.json")).unwrap();
        let delegations = response.result;

        let result = map_balance_staking(delegations);

        assert!(result.is_some());
        let balance = result.unwrap();
        assert_eq!(balance.asset_id.chain, Chain::Sui);

        // Total staked: sum of all principal + estimated_reward values
        assert_eq!(balance.balance.staked, BigUint::from(9113484503_u64));
        assert_eq!(balance.balance.available, BigUint::from(0u32));
    }

    #[test]
    fn test_map_balance_staking_empty() {
        let delegations: Vec<SuiStakeDelegation> = vec![];
        let result = map_balance_staking(delegations);

        assert!(result.is_none());
    }
}

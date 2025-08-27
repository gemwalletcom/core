use crate::models::rpc::Balance as SuiBalance;
use crate::models::staking::SuiStakeDelegation;
use crate::{SUI_COIN_TYPE, SUI_COIN_TYPE_FULL};
use primitives::{AssetBalance, AssetId, Balance, Chain};

pub fn map_balance_coin(balance: SuiBalance) -> AssetBalance {
    AssetBalance::new_balance(Chain::Sui.as_asset_id(), Balance::coin_balance(balance.total_balance.to_string()))
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

            AssetBalance::new_balance(AssetId::from_token(Chain::Sui, &token_id), Balance::coin_balance(balance.to_string()))
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
        Balance::stake_balance(staked.to_string(), "0".to_string(), None),
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
        Balance::stake_balance(staked_total.to_string(), "0".to_string(), None),
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

            asset_id.map(|asset_id| AssetBalance::new_balance(asset_id, Balance::coin_balance(balance.total_balance.to_string())))
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
        assert_eq!(result.balance.available, "52855428706");
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
        assert_eq!(result[0].balance.available, "3685298"); // USDC balance
        assert_eq!(result[1].balance.available, "1000"); // TOKEN balance
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
        assert_eq!(balance.balance.staked, "9113484503");
        assert_eq!(balance.balance.available, "0");
    }

    #[test]
    fn test_map_balance_staking_empty() {
        let delegations: Vec<SuiStakeDelegation> = vec![];
        let result = map_balance_staking(delegations);
        
        assert!(result.is_none());
    }
}

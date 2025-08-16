use primitives::{AssetBalance, Balance, Chain, AssetId};
use crate::rpc::model::Account;

pub fn map_balance_coin(account: &Account, chain: Chain) -> AssetBalance {
    let (available, reserved): (i64, i64) = {
        let amount = account.amount;
        if amount > 0 {
            let reserved = account.min_balance;
            (
                std::cmp::max(amount - reserved, 0),
                reserved
            )
        } else {
            (0, 0)
        }
    };
    
    AssetBalance::new_with_active(
        chain.as_asset_id(),
        Balance::with_reserved(available.to_string(), reserved.to_string()),
        true,
    )
}

pub fn map_balance_tokens(account: &Account, token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    token_ids.into_iter().map(|token_id| {
        let (balance, is_active): (i64, bool) = {
            if let Some(asset) = account.assets.iter().find(|asset| asset.asset_id.to_string() == token_id) {
                (asset.amount, true)
            } else {
                (0, false)
            }
        };
        
        AssetBalance::new_with_active(
            AssetId {
                chain,
                token_id: Some(token_id),
            },
            Balance::coin_balance(balance.to_string()),
            is_active,
        )
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{Account, Asset};
    use primitives::Chain;

    #[test]
    fn test_map_balance_coin_sufficient_balance() {
        let account = Account { amount: 1000000, min_balance: 100000, assets: vec![] };
        let balance = map_balance_coin(&account, Chain::Algorand);
        
        assert_eq!(balance.balance.available, "900000");
        assert_eq!(balance.balance.reserved, "100000");
        assert_eq!(balance.is_active, Some(true));
    }

    #[test]
    fn test_map_balance_coin_insufficient_balance() {
        let account = Account { amount: 50000, min_balance: 100000, assets: vec![] };
        let balance = map_balance_coin(&account, Chain::Algorand);
        
        assert_eq!(balance.balance.available, "0");
        assert_eq!(balance.balance.reserved, "100000");
    }

    #[test]
    fn test_map_balance_coin_zero_balance() {
        let account = Account { amount: 0, min_balance: 100000, assets: vec![] };
        let balance = map_balance_coin(&account, Chain::Algorand);
        
        assert_eq!(balance.balance.available, "0");
        assert_eq!(balance.balance.reserved, "0");
    }

    #[test]
    fn test_map_balance_tokens_with_assets() {
        let account = Account {
            amount: 1000000,
            min_balance: 100000,
            assets: vec![Asset { asset_id: 123456, amount: 5000 }],
        };
        
        let token_ids = vec!["123456".to_string(), "999999".to_string()];
        let balances = map_balance_tokens(&account, token_ids, Chain::Algorand);
        
        assert_eq!(balances.len(), 2);
        assert_eq!(balances[0].balance.available, "5000");
        assert_eq!(balances[0].is_active, Some(true));
        assert_eq!(balances[1].balance.available, "0");
        assert_eq!(balances[1].is_active, Some(false));
    }

    #[test]
    fn test_map_balance_tokens_empty_account() {
        let account = Account { amount: 0, min_balance: 100000, assets: vec![] };
        let token_ids = vec!["123456".to_string()];
        let balances = map_balance_tokens(&account, token_ids, Chain::Algorand);
        
        assert_eq!(balances.len(), 1);
        assert_eq!(balances[0].balance.available, "0");
        assert_eq!(balances[0].is_active, Some(false));
    }
}
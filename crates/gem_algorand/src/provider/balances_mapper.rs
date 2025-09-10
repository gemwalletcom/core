use crate::models::{Account, Asset};
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Balance, Chain};

pub fn map_balance_coin(account: &Account, chain: Chain) -> AssetBalance {
    let (available, reserved): (u64, u64) = {
        let amount = account.amount;
        if amount > 0 {
            let reserved = account.min_balance;
            (std::cmp::max(amount - reserved, 0), reserved)
        } else {
            (0, 0)
        }
    };

    AssetBalance::new_with_active(
        chain.as_asset_id(),
        Balance::with_reserved(BigUint::from(available), BigUint::from(reserved)),
        true,
    )
}

pub fn map_balance_tokens(account: &Account, token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    token_ids
        .into_iter()
        .map(|token_id| {
            let (balance, is_active): (u64, bool) = {
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
                Balance::coin_balance(BigUint::from(balance)),
                is_active,
            )
        })
        .collect()
}

pub fn map_assets_balance(assets: Vec<Asset>) -> Vec<AssetBalance> {
    assets
        .into_iter()
        .map(|asset| {
            AssetBalance::new(
                AssetId::from_token(Chain::Algorand, &asset.asset_id.to_string()),
                BigUint::from(asset.amount.max(0) as u64),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::Account;
    use primitives::Chain;

    #[test]
    fn test_map_balance_coin() {
        let account: Account = serde_json::from_str(include_str!("../../testdata/account.json")).unwrap();
        let balance = map_balance_coin(&account, Chain::Algorand);

        assert_eq!(balance.balance.available, BigUint::from(71414422_u64));
        assert_eq!(balance.balance.reserved, BigUint::from(200000_u64));
        assert!(balance.is_active);
    }
}

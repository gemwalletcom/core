use crate::models::rpc::{Account, Asset};
use primitives::{AssetBalance, AssetId, Balance, Chain};

pub fn map_balance_coin(account: &Account, chain: Chain) -> AssetBalance {
    let (available, reserved): (i64, i64) = {
        let amount = account.amount;
        if amount > 0 {
            let reserved = account.min_balance;
            (std::cmp::max(amount - reserved, 0), reserved)
        } else {
            (0, 0)
        }
    };

    AssetBalance::new_with_active(chain.as_asset_id(), Balance::with_reserved(available.to_string(), reserved.to_string()), true)
}

pub fn map_balance_tokens(account: &Account, token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    token_ids
        .into_iter()
        .map(|token_id| {
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
        })
        .collect()
}

pub fn map_assets_balance(assets: Vec<Asset>) -> Vec<AssetBalance> {
    assets
        .into_iter()
        .map(|asset| AssetBalance::new(AssetId::from_token(Chain::Algorand, &asset.asset_id.to_string()), asset.amount.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rpc::Account;
    use primitives::Chain;

    #[test]
    fn test_map_balance_coin() {
        let account: Account = serde_json::from_str(include_str!("../../testdata/account.json")).unwrap();
        let balance = map_balance_coin(&account, Chain::Algorand);

        assert_eq!(balance.balance.available, "72516422");
        assert_eq!(balance.balance.reserved, "200000");
        assert_eq!(balance.is_active, Some(true));
    }
}

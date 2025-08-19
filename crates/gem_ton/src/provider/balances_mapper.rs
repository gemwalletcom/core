use crate::rpc::model::{AccountInfo, JettonWalletsResponse};
use primitives::{AssetBalance, AssetId, Chain};

pub fn map_coin_balance(account: AccountInfo) -> AssetBalance {
    AssetBalance::new(Chain::Ton.as_asset_id(), account.balance.to_string())
}

pub fn map_balance_tokens(wallets: JettonWalletsResponse, token_ids: Vec<String>) -> Vec<AssetBalance> {
    wallets
        .jetton_wallets
        .into_iter()
        .filter_map(|wallet| {
            let jetton_token_id = crate::address::hex_to_base64_address(wallet.jetton).ok()?;

            if token_ids.iter().any(|token_id| token_id == &jetton_token_id) {
                Some(AssetBalance::new(AssetId::from_token(Chain::Ton, &jetton_token_id), wallet.balance))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::AccountInfo;

    #[test]
    fn test_map_balance_coin() {
        let account_info = AccountInfo {
            balance: 62709394797,
            status: "active".to_string(),
        };

        let result = map_coin_balance(account_info);

        assert_eq!(result.asset_id, Chain::Ton.as_asset_id());
        assert_eq!(result.balance.available, "62709394797");
    }

    #[test]
    fn test_map_balance_tokens() {
        let response: JettonWalletsResponse = serde_json::from_str(include_str!("../../testdata/balance_jettons.json")).unwrap();

        let token_id = "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs";
        let token_ids = vec![token_id.to_string()];
        let result = map_balance_tokens(response, token_ids);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].asset_id, AssetId::from_token(Chain::Ton, token_id));
        assert_eq!(result[0].balance.available, "3201565");
    }
}

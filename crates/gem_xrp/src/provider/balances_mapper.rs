use crate::{
    XRP_DEFAULT_ASSET_DECIMALS,
    models::rpc::{AccountInfo, AccountObjects},
};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, AssetId, Balance, Chain};
use std::{collections::HashMap, error::Error};

pub fn map_balance_coin(account: Option<AccountInfo>, asset_id: AssetId, reserved_amount: u64) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let available = if let Some(account) = account {
        account.balance.saturating_sub(reserved_amount)
    } else {
        0
    };

    Ok(AssetBalance::new_balance(
        asset_id,
        Balance::with_reserved(BigUint::from(available), BigUint::from(reserved_amount)),
    ))
}

fn account_objects_to_balances(objects: &AccountObjects, chain: Chain) -> Vec<AssetBalance> {
    objects
        .account_objects
        .as_ref()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|obj| {
            if obj.high_limit.currency.len() <= 3 {
                return None;
            }

            let value = BigNumberFormatter::value_from_amount_biguint(&obj.balance.value, XRP_DEFAULT_ASSET_DECIMALS).unwrap_or_default();
            let is_active = value > BigUint::from(0u32);
            let asset_id = AssetId::from_token(chain, &obj.high_limit.issuer);
            let balance = Balance::coin_balance(value);

            Some(AssetBalance::new_with_active(asset_id, balance, is_active))
        })
        .collect()
}

pub fn map_balance_tokens(objects: &AccountObjects, token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    let available_balances: HashMap<String, AssetBalance> = account_objects_to_balances(objects, chain)
        .into_iter()
        .filter_map(|x| x.asset_id.token_id.clone().map(|token_id| (token_id, x)))
        .collect();

    token_ids
        .into_iter()
        .map(|token_id| {
            available_balances.get(&token_id).cloned().unwrap_or_else(|| {
                let asset_id = AssetId::from_token(chain, &token_id);
                let balance = Balance::coin_balance(BigUint::from(0u32));
                AssetBalance::new_with_active(asset_id, balance, false)
            })
        })
        .collect()
}

pub fn map_balance_assets(objects: &AccountObjects, chain: Chain) -> Vec<AssetBalance> {
    account_objects_to_balances(objects, chain)
        .into_iter()
        .filter(|x| x.balance.available > BigUint::from(0u32))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::result::XRPResult;
    use crate::models::rpc::AccountInfo;
    use primitives::{AssetId, Chain};

    #[test]
    fn test_map_native_balance() {
        let account = AccountInfo {
            balance: 10000000, // 10 XRP
            sequence: 100,
            owner_count: 2,
            account: None,
            flags: None,
            ledger_entry_type: None,
        };

        let asset_id = AssetId::from_chain(Chain::Xrp);
        let reserved_amount = 1000000; // 1 XRP reserve

        let result = map_balance_coin(Some(account), asset_id.clone(), reserved_amount).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, BigUint::from(9000000_u64)); // 10 - 1 = 9 XRP
        assert_eq!(result.balance.reserved, BigUint::from(1000000_u64));
    }

    #[test]
    fn test_map_native_balance_insufficient() {
        let account = AccountInfo {
            balance: 500000, // 0.5 XRP
            sequence: 100,
            owner_count: 2,
            account: None,
            flags: None,
            ledger_entry_type: None,
        };

        let asset_id = AssetId::from_chain(Chain::Xrp);
        let reserved_amount = 1000000; // 1 XRP reserve

        let result = map_balance_coin(Some(account), asset_id.clone(), reserved_amount).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, BigUint::from(0u32)); // Insufficient balance
        assert_eq!(result.balance.reserved, BigUint::from(1000000_u64));
    }

    #[test]
    fn test_map_balance_tokens() {
        let response: XRPResult<AccountObjects> = serde_json::from_str(include_str!("../testdata/accounts_objects_tokens.json")).unwrap();
        let account_objects = response.result;

        let token_ids = vec!["rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De".to_string()];

        let result = map_balance_tokens(&account_objects, token_ids, Chain::Xrp);

        assert_eq!(result.len(), 1);

        let balance = &result[0];
        assert_eq!(balance.asset_id, AssetId::from_token(Chain::Xrp, "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"));
        assert_eq!(balance.balance.available, BigUint::from(171000000000000_u64));
        assert!(balance.is_active);
    }

    #[test]
    fn test_map_balance_assets() {
        let response: XRPResult<AccountObjects> = serde_json::from_str(include_str!("../testdata/accounts_objects_tokens.json")).unwrap();
        let account_objects = response.result;

        let result = map_balance_assets(&account_objects, Chain::Xrp);

        assert!(!result.is_empty());
        for balance in &result {
            assert_eq!(balance.asset_id.chain, Chain::Xrp);
            assert!(balance.asset_id.token_id.is_some());
            assert!(balance.balance.available > BigUint::from(0u32));
            assert!(balance.is_active);
        }
    }
}

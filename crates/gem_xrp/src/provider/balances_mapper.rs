use crate::models::rpc::{AccountInfo, AccountObjects};
use primitives::{AssetBalance, AssetId, Balance, Chain};
use std::error::Error;

pub fn map_native_balance(account: &AccountInfo, asset_id: AssetId, reserved_amount: u64) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let balance_str = &account.balance;
    let balance: u64 = balance_str.parse().map_err(|_| "Invalid balance format")?;

    let available = if balance > reserved_amount {
        (balance - reserved_amount).to_string()
    } else {
        "0".to_string()
    };

    Ok(AssetBalance::new_balance(
        asset_id,
        Balance::with_reserved(available, reserved_amount.to_string()),
    ))
}

pub fn map_token_balances(objects: &AccountObjects, token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    let mut balances = Vec::new();
    for token_id in token_ids {
        if let Some(object) = objects
            .account_objects
            .iter()
            .find(|obj| obj.high_limit.issuer == token_id && obj.high_limit.currency.len() > 3)
        {
            let asset_id = AssetId::from_token(chain, &token_id);
            let balance = Balance::coin_balance(object.balance.value.clone());
            balances.push(AssetBalance::new_with_active(asset_id, balance, true));
        } else {
            let asset_id = AssetId::from_token(chain, &token_id);
            let balance = Balance::coin_balance("0".to_string());
            balances.push(AssetBalance::new_with_active(asset_id, balance, false));
        }
    }
    balances
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rpc::AccountInfo;
    use primitives::{AssetId, Chain};

    #[test]
    fn test_map_native_balance() {
        let account = AccountInfo {
            balance: "10000000".to_string(), // 10 XRP
            sequence: 100,
            owner_count: 2,
            account: None,
            flags: None,
            ledger_entry_type: None,
        };

        let asset_id = AssetId::from_chain(Chain::Xrp);
        let reserved_amount = 1000000; // 1 XRP reserve

        let result = map_native_balance(&account, asset_id.clone(), reserved_amount).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, "9000000"); // 10 - 1 = 9 XRP
        assert_eq!(result.balance.reserved, "1000000");
    }

    #[test]
    fn test_map_native_balance_insufficient() {
        let account = AccountInfo {
            balance: "500000".to_string(), // 0.5 XRP
            sequence: 100,
            owner_count: 2,
            account: None,
            flags: None,
            ledger_entry_type: None,
        };

        let asset_id = AssetId::from_chain(Chain::Xrp);
        let reserved_amount = 1000000; // 1 XRP reserve

        let result = map_native_balance(&account, asset_id.clone(), reserved_amount).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, "0"); // Insufficient balance
        assert_eq!(result.balance.reserved, "1000000");
    }
}

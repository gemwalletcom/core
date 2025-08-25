use crate::models::account::Account;
use primitives::{AssetBalance, Chain};
use std::error::Error;

pub fn map_native_balance(account: &Account) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    Ok(AssetBalance::new(Chain::Near.as_asset_id(), account.amount.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::Account;

    #[test]
    fn test_map_native_balance() {
        let account = Account {
            amount: "1000000000000000000000000".to_string(),
        };

        let result = map_native_balance(&account).unwrap();

        assert_eq!(result.asset_id, Chain::Near.as_asset_id());
        assert_eq!(result.balance.available, "1000000000000000000000000");
    }
}

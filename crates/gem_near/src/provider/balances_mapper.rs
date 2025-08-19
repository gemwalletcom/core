use crate::models::account::NearAccount;
use primitives::{AssetBalance, Chain};
use std::error::Error;

pub fn map_native_balance(account: &NearAccount) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    Ok(AssetBalance::new(Chain::Near.as_asset_id(), account.amount.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::NearAccount;

    #[test]
    fn test_map_native_balance() {
        let account = NearAccount {
            amount: "1000000000000000000000000".to_string(),
        };

        let result = map_native_balance(&account).unwrap();

        assert_eq!(result.asset_id, Chain::Near.as_asset_id());
        assert_eq!(result.balance.available, "1000000000000000000000000");
    }
}

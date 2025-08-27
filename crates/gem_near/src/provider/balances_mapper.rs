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
    use num_bigint::BigUint;

    #[test]
    fn test_map_native_balance() {
        let account = Account {
            amount: BigUint::from(1000000000000000000000000_u128),
        };

        let result = map_native_balance(&account).unwrap();

        assert_eq!(result.asset_id, Chain::Near.as_asset_id());
        assert_eq!(result.balance.available, BigUint::from(1000000000000000000000000_u128));
    }
}

use crate::models::account::NearAccount;
use primitives::{AssetBalance, AssetId, Chain};
use std::error::Error;

pub fn map_native_balance(account: &NearAccount, asset_id: AssetId, _chain: Chain) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    Ok(AssetBalance::new(asset_id, account.amount.clone()))
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
        let chain = Chain::Near;
        let asset_id = AssetId::from_chain(chain);

        let result = map_native_balance(&account, asset_id.clone(), chain).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, "1000000000000000000000000");
    }
}

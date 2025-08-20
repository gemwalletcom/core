use primitives::{AssetBalance, AssetId, Chain};
use std::error::Error;

use crate::models::TronAccount;

pub fn map_coin_balance(account: &TronAccount) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let available_balance = account.balance.unwrap_or(0).to_string();
    
    Ok(AssetBalance::new(
        AssetId::from_chain(Chain::Tron),
        available_balance,
    ))
}


pub fn map_token_balance(
    balance_hex: &str,
    asset_id: AssetId,
) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let balance = if balance_hex.is_empty() || balance_hex == "0x" {
        "0".to_string()
    } else {
        let hex_str = balance_hex.strip_prefix("0x").unwrap_or(balance_hex);
        u128::from_str_radix(hex_str, 16)
            .map_err(|e| format!("Failed to parse hex balance: {}", e))?
            .to_string()
    };

    Ok(AssetBalance::new(asset_id, balance))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TronAccount, TronSmartContractResult};
    use primitives::{AssetId, Chain};
    use serde_json;

    #[test]
    fn test_map_coin_balance_with_real_payload() {
        let account: TronAccount = serde_json::from_str(include_str!("../../testdata/balance_coin.json")).unwrap();
        let balance = map_coin_balance(&account).unwrap();
        
        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.available, "2928601454");
    }

    #[test]
    fn test_map_token_balance_with_real_payload() {
        let response: TronSmartContractResult = serde_json::from_str(include_str!("../../testdata/balance_token.json")).unwrap();
        let asset_id = AssetId::from(Chain::Tron, Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()));
        let balance = map_token_balance(&response.constant_result[0], asset_id.clone()).unwrap();
        
        assert_eq!(balance.asset_id, asset_id);
        assert_eq!(balance.balance.available, "136389002");
    }

    #[test]
    fn test_map_token_balance_edge_cases() {
        let asset_id = AssetId::from(Chain::Tron, Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()));
        
        let balance = map_token_balance("", asset_id.clone()).unwrap();
        assert_eq!(balance.balance.available, "0");

        let balance = map_token_balance("0x", asset_id.clone()).unwrap();
        assert_eq!(balance.balance.available, "0");

        let balance = map_token_balance("0x0", asset_id.clone()).unwrap();
        assert_eq!(balance.balance.available, "0");

        let balance = map_token_balance("0x821218a", asset_id).unwrap();
        assert_eq!(balance.balance.available, "136389002");
    }

    #[test]
    fn test_map_coin_balance_zero_balance() {
        let account = TronAccount {
            balance: None,
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            active_permission: None,
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        };
        
        let balance = map_coin_balance(&account).unwrap();
        assert_eq!(balance.balance.available, "0");
    }
}
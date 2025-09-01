use num_bigint::BigUint;
use num_traits::Num;
use primitives::{AssetBalance, AssetId, Chain};
use std::error::Error;

use crate::models::{TronAccount, TronReward};

pub fn map_coin_balance(account: &TronAccount) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let available_balance = BigUint::from(account.balance.unwrap_or(0));
    Ok(AssetBalance::new(AssetId::from_chain(Chain::Tron), available_balance))
}

pub fn map_token_balance(balance_hex: &str, asset_id: AssetId) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let balance = if balance_hex.is_empty() || balance_hex == "0x" {
        BigUint::from(0u32)
    } else {
        let hex_str = balance_hex.strip_prefix("0x").unwrap_or(balance_hex);
        BigUint::from_str_radix(hex_str, 16).map_err(|e| format!("Failed to parse hex balance: {}", e))?
    };

    Ok(AssetBalance::new(asset_id, balance))
}

pub fn map_staking_balance(account: &TronAccount, reward: &TronReward) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let staked_amount = account
        .frozen_v2
        .as_ref()
        .map(|frozen_list| frozen_list.iter().map(|frozen| frozen.amount.unwrap_or(0)).sum::<u64>())
        .unwrap_or(0);

    let pending_amount = account
        .unfrozen_v2
        .as_ref()
        .map(|unfrozen_list| unfrozen_list.iter().map(|unfrozen| unfrozen.unfreeze_amount.unwrap_or(0)).sum::<u64>())
        .unwrap_or(0);

    let rewards_amount = reward.reward;

    Ok(AssetBalance::new_staking(
        AssetId::from_chain(Chain::Tron),
        BigUint::from(staked_amount),
        BigUint::from(pending_amount),
        BigUint::from(rewards_amount),
    ))
}

pub(crate) fn format_address_parameter(address: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let owner_bytes = bs58::decode(address)
        .into_vec()
        .map_err(|e| format!("Invalid owner address {}: {}", address, e))?;

    if owner_bytes.len() != 25 || owner_bytes[0] != 0x41 {
        return Err(format!("Invalid TRON address format: {}", address).into());
    }

    let address_bytes = &owner_bytes[1..21];
    Ok(format!("{:0>64}", hex::encode(address_bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TronAccount, TronFrozen, TronReward, TronSmartContractResult, TronUnfrozen};
    use primitives::{AssetId, Chain};
    use serde_json;

    #[test]
    fn test_map_coin_balance_with_real_payload() {
        let account: TronAccount = serde_json::from_str(include_str!("../../testdata/balance_coin.json")).unwrap();
        let balance = map_coin_balance(&account).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.available, BigUint::from(2928601454_u64));
    }

    #[test]
    fn test_map_token_balance_with_real_payload() {
        let response: TronSmartContractResult = serde_json::from_str(include_str!("../../testdata/balance_token.json")).unwrap();
        let asset_id = AssetId::from(Chain::Tron, Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()));
        let balance = map_token_balance(&response.constant_result[0], asset_id.clone()).unwrap();

        assert_eq!(balance.asset_id, asset_id);
        assert_eq!(balance.balance.available, BigUint::from(136389002_u64));
    }

    #[test]
    fn test_map_token_balance_edge_cases() {
        let asset_id = AssetId::from(Chain::Tron, Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()));

        let balance = map_token_balance("", asset_id.clone()).unwrap();
        assert_eq!(balance.balance.available, BigUint::from(0u32));

        let balance = map_token_balance("0x", asset_id.clone()).unwrap();
        assert_eq!(balance.balance.available, BigUint::from(0u32));

        let balance = map_token_balance("0x0", asset_id.clone()).unwrap();
        assert_eq!(balance.balance.available, BigUint::from(0u32));

        let balance = map_token_balance("0x821218a", asset_id).unwrap();
        assert_eq!(balance.balance.available, BigUint::from(136389002_u64));
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
        assert_eq!(balance.balance.available, BigUint::from(0u32));
    }

    #[test]
    fn test_format_address_parameter() {
        let address = "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb";
        let parameter = format_address_parameter(address).unwrap();
        assert_eq!(parameter, "0000000000000000000000002e1d447fa4169390cf5f5b3d12d380decfbfe20f");
    }

    #[test]
    fn test_map_staking_balance() {
        let account = TronAccount {
            balance: Some(1000),
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            active_permission: None,
            votes: None,
            frozen_v2: Some(vec![
                TronFrozen {
                    frozen_type: Some("BANDWIDTH".to_string()),
                    amount: Some(5000000),
                },
                TronFrozen {
                    frozen_type: Some("ENERGY".to_string()),
                    amount: Some(3000000),
                },
            ]),
            unfrozen_v2: Some(vec![TronUnfrozen {
                unfreeze_amount: Some(2000000),
                unfreeze_expire_time: Some(1234567890),
            }]),
        };

        let reward = TronReward { reward: 100000 };

        let balance = map_staking_balance(&account, &reward).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.staked, BigUint::from(8000000_u64));
        assert_eq!(balance.balance.pending, BigUint::from(2000000_u64));
        assert_eq!(balance.balance.rewards, BigUint::from(100000_u64));
    }

    #[test]
    fn test_map_staking_balance_empty_fields() {
        let account = TronAccount {
            balance: Some(1000),
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            active_permission: None,
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        };

        let reward = TronReward { reward: 0 };

        let balance = map_staking_balance(&account, &reward).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.staked, BigUint::from(0_u64));
        assert_eq!(balance.balance.pending, BigUint::from(0_u64));
        assert_eq!(balance.balance.rewards, BigUint::from(0_u64));
    }
}

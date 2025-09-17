use num_bigint::BigUint;
use num_traits::Num;
use primitives::{asset_balance::{Balance, BalanceMetadata}, AssetBalance, AssetId, Chain};
use std::error::Error;

use crate::models::{TronAccount, TronAccountUsage, TronReward};

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

pub fn map_staking_balance(account: &TronAccount, reward: &TronReward, usage: &TronAccountUsage) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let (bandwidth_frozen, energy_frozen) = account
        .frozen_v2
        .as_ref()
        .map(|frozen_list| {
            let mut bandwidth = 0u64;
            let mut energy = 0u64;

            for frozen in frozen_list {
                let amount = frozen.amount.unwrap_or(0);
                match frozen.frozen_type.as_deref() {
                    Some("ENERGY") => energy += amount,
                    _ => bandwidth += amount, // None or "BANDWIDTH" defaults to bandwidth
                }
            }
            (bandwidth, energy)
        })
        .unwrap_or((0, 0));

    let voted_amount = account
        .votes
        .as_ref()
        .map(|votes| votes.iter().map(|vote| vote.vote_count * 10_u64.pow(6)).sum::<u64>())
        .unwrap_or(0);

    let pending_amount = account
        .unfrozen_v2
        .as_ref()
        .map(|unfrozen_list| unfrozen_list.iter().map(|unfrozen| unfrozen.unfreeze_amount.unwrap_or(0)).sum::<u64>())
        .unwrap_or(0);

    let rewards_amount = reward.reward;

    let energy_total = usage.energy_limit.unwrap_or(0);
    let energy_available = energy_total - usage.energy_used.unwrap_or(0);

    let bandwidth_total = usage.free_net_limit.unwrap_or(0) + usage.net_limit.unwrap_or(0);
    let bandwidth_available =
        (usage.free_net_limit.unwrap_or(0) - usage.free_net_used.unwrap_or(0)) + (usage.net_limit.unwrap_or(0) - usage.net_used.unwrap_or(0));

    let metadata = BalanceMetadata {
        energy_available,
        energy_total,
        bandwidth_available,
        bandwidth_total,
    };

    Ok(AssetBalance::new_balance(
        AssetId::from_chain(Chain::Tron),
        create_tron_stake_balance(
            BigUint::from(bandwidth_frozen),
            BigUint::from(energy_frozen),
            BigUint::from(voted_amount),
            BigUint::from(pending_amount),
            BigUint::from(rewards_amount),
            metadata,
        ),
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

fn create_tron_stake_balance(
    frozen: BigUint, // bandwidth frozen
    locked: BigUint, // energy frozen
    staked: BigUint, // voted amount
    pending: BigUint, // unfreezing amount
    rewards: BigUint, // voting rewards
    metadata: BalanceMetadata,
) -> Balance {
    Balance {
        available: BigUint::from(0u32),
        frozen,
        locked,
        staked,
        pending,
        rewards,
        reserved: BigUint::from(0u32),
        withdrawable: BigUint::from(0u32),
        metadata: Some(metadata),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TronAccount, TronFrozen, TronReward, TronSmartContractResult, TronUnfrozen, TronVote};
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
        let usage = TronAccountUsage {
            energy_limit: Some(1000000),
            energy_used: Some(500000),
            free_net_limit: Some(1000000),
            free_net_used: Some(500000),
            net_used: Some(200000),
            net_limit: Some(1000000),
        };

        let balance = map_staking_balance(&account, &reward, &usage).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.frozen, BigUint::from(5000000_u64));
        assert_eq!(balance.balance.locked, BigUint::from(3000000_u64));
        assert_eq!(balance.balance.staked, BigUint::from(0_u64));
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
        let usage = TronAccountUsage {
            energy_limit: Some(1000000),
            energy_used: Some(500000),
            free_net_limit: Some(1000000),
            free_net_used: Some(500000),
            net_used: Some(200000),
            net_limit: Some(1000000),
        };
        let balance = map_staking_balance(&account, &reward, &usage).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.frozen, BigUint::from(0_u64));
        assert_eq!(balance.balance.locked, BigUint::from(0_u64));
        assert_eq!(balance.balance.staked, BigUint::from(0_u64));
        assert_eq!(balance.balance.pending, BigUint::from(0_u64));
        assert_eq!(balance.balance.rewards, BigUint::from(0_u64));
    }

    #[test]
    fn test_map_staking_balance_with_votes() {
        let account = TronAccount {
            balance: Some(1000),
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            active_permission: None,
            votes: Some(vec![
                TronVote {
                    vote_address: "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH".to_string(),
                    vote_count: 3000000,
                },
                TronVote {
                    vote_address: "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1".to_string(),
                    vote_count: 2000000,
                },
            ]),
            frozen_v2: Some(vec![TronFrozen {
                frozen_type: Some("BANDWIDTH".to_string()),
                amount: Some(8000000),
            }]),
            unfrozen_v2: None,
        };

        let reward = TronReward { reward: 50000 };
        let usage = TronAccountUsage {
            energy_limit: Some(1000000),
            energy_used: Some(500000),
            free_net_limit: Some(1000000),
            free_net_used: Some(500000),
            net_used: Some(200000),
            net_limit: Some(1000000),
        };

        let balance = map_staking_balance(&account, &reward, &usage).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.frozen, BigUint::from(8000000_u64));
        assert_eq!(balance.balance.locked, BigUint::from(0_u64));
        assert_eq!(balance.balance.staked, BigUint::from(5000000000000_u64));
        assert_eq!(balance.balance.pending, BigUint::from(0_u64));
        assert_eq!(balance.balance.rewards, BigUint::from(50000_u64));
    }

    #[test]
    fn test_map_staking_balance_metadata() {
        let account = TronAccount {
            balance: Some(1000),
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            active_permission: None,
            votes: None,
            frozen_v2: Some(vec![TronFrozen {
                frozen_type: Some("ENERGY".to_string()),
                amount: Some(1000000),
            }]),
            unfrozen_v2: None,
        };

        let reward = TronReward { reward: 50000 };
        let usage = TronAccountUsage {
            energy_limit: Some(2000000),
            energy_used: Some(800000),
            free_net_limit: Some(1500),
            free_net_used: Some(500),
            net_used: Some(0),
            net_limit: Some(5000),
        };

        let balance = map_staking_balance(&account, &reward, &usage).unwrap();
        let metadata = balance.balance.metadata.as_ref().unwrap();

        assert_eq!(metadata.energy_available, 1200000);
        assert_eq!(metadata.energy_total, 2000000);
        assert_eq!(metadata.bandwidth_available, 6000);
        assert_eq!(metadata.bandwidth_total, 6500);
    }

    #[test]
    fn test_create_tron_stake_balance() {
        let metadata = BalanceMetadata {
            energy_available: 1000,
            energy_total: 2000,
            bandwidth_available: 500,
            bandwidth_total: 1000,
        };

        let balance = create_tron_stake_balance(
            BigUint::from(100_u64),
            BigUint::from(200_u64),
            BigUint::from(300_u64),
            BigUint::from(400_u64),
            BigUint::from(500_u64),
            metadata.clone(),
        );

        assert_eq!(balance.available, BigUint::from(0_u32));
        assert_eq!(balance.frozen, BigUint::from(100_u64));
        assert_eq!(balance.locked, BigUint::from(200_u64));
        assert_eq!(balance.staked, BigUint::from(300_u64));
        assert_eq!(balance.pending, BigUint::from(400_u64));
        assert_eq!(balance.rewards, BigUint::from(500_u64));
        assert_eq!(balance.reserved, BigUint::from(0_u32));
        assert_eq!(balance.withdrawable, BigUint::from(0_u32));
        assert_eq!(balance.metadata, Some(metadata));
    }

    #[test]
    fn test_map_staking_balance_metadata_with_none_values() {
        let account = TronAccount {
            balance: None,
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            active_permission: None,
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        };

        let reward = TronReward { reward: 0 };
        let usage = TronAccountUsage {
            energy_limit: None,
            energy_used: None,
            free_net_limit: None,
            free_net_used: None,
            net_used: None,
            net_limit: None,
        };

        let balance = map_staking_balance(&account, &reward, &usage).unwrap();
        let metadata = balance.balance.metadata.as_ref().unwrap();

        assert_eq!(metadata.energy_available, 0);
        assert_eq!(metadata.energy_total, 0);
        assert_eq!(metadata.bandwidth_available, 0);
        assert_eq!(metadata.bandwidth_total, 0);
    }
}

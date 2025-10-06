use num_bigint::BigUint;
use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator};

use crate::models::{DelegationPoolStake, ValidatorInfo, ValidatorSet};

pub fn map_validators(validator_set: ValidatorSet, apy: f64, pool_address: &str, commission: f64) -> Vec<DelegationValidator> {
    validator_set
        .active_validators
        .iter()
        .filter(|v| v.addr == pool_address)
        .map(|v| map_validator(v, apy, commission, true))
        .collect()
}

pub fn map_validator(validator: &ValidatorInfo, apy: f64, commission: f64, is_active: bool) -> DelegationValidator {
    DelegationValidator {
        chain: Chain::Aptos,
        id: validator.addr.clone(),
        name: "".to_string(),
        is_active,
        commission,
        apr: apy,
    }
}

fn map_delegation(
    asset_id: &primitives::AssetId,
    state: DelegationState,
    balance: BigUint,
    state_name: &str,
    validator_id: &str,
) -> DelegationBase {
    DelegationBase {
        asset_id: asset_id.clone(),
        state,
        balance,
        shares: BigUint::from(0u32),
        rewards: BigUint::from(0u32),
        completion_date: None,
        delegation_id: format!("{}_{}", state_name, validator_id),
        validator_id: validator_id.to_string(),
    }
}

pub fn map_delegations(stakes: Vec<(String, DelegationPoolStake)>) -> Vec<DelegationBase> {
    let asset_id = Chain::Aptos.as_asset_id();

    stakes
        .into_iter()
        .flat_map(|(pool_address, stake)| {
            let mut delegations = Vec::new();

            if stake.active > BigUint::from(0u32) {
                delegations.push(map_delegation(&asset_id, DelegationState::Active, stake.active, "active", &pool_address));
            }

            if stake.pending > BigUint::from(0u32) {
                delegations.push(map_delegation(&asset_id, DelegationState::Pending, stake.pending, "pending", &pool_address));
            }

            if stake.inactive > BigUint::from(0u32) {
                delegations.push(map_delegation(&asset_id, DelegationState::AwaitingWithdrawal, stake.inactive, "inactive", &pool_address));
            }

            delegations
        })
        .collect()
}

pub fn calculate_apy(staking_config: &crate::models::StakingConfig) -> f64 {
    if staking_config.rewards_rate_denominator == 0 {
        return 0.0;
    }

    let epoch_rewards_rate = staking_config.rewards_rate as f64 / staking_config.rewards_rate_denominator as f64;
    let epochs_per_year = 365.25 * 24.0 * 60.0 * 60.0 / 7200.0;

    epoch_rewards_rate * epochs_per_year * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StakingConfig;

    #[test]
    fn test_calculate_apy() {
        let config = StakingConfig {
            rewards_rate: 16000000000000,
            rewards_rate_denominator: 100000000000000000,
        };

        let apy = calculate_apy(&config);
        println!("APY: {}", apy);
        println!("Diff: {}", (apy - 7.004472300000001).abs());

        assert!((apy - 70.128).abs() < 0.01);
    }

    #[test]
    fn test_calculate_apy_zero_denominator() {
        let config = StakingConfig {
            rewards_rate: 1600000000000000,
            rewards_rate_denominator: 0,
        };

        let apy = calculate_apy(&config);

        assert_eq!(apy, 0.0);
    }
}

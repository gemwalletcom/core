use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator};

use crate::models::{DelegationPoolStake, ReconfigurationState, StakingConfig, ValidatorInfo, ValidatorSet};

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
    validator_id: &str,
    completion_date: Option<DateTime<Utc>>,
) -> DelegationBase {
    DelegationBase {
        asset_id: asset_id.clone(),
        state,
        balance,
        shares: BigUint::from(0u32),
        rewards: BigUint::from(0u32),
        completion_date,
        delegation_id: format!("{}_{}", state.as_ref().to_lowercase(), validator_id),
        validator_id: validator_id.to_string(),
    }
}

const EPOCH_DURATION_SECS: u64 = 7200;

fn calculate_pending_completion_date(reconfig: &ReconfigurationState) -> Option<DateTime<Utc>> {
    let now_micros = Utc::now().timestamp_micros() as u64;
    let last_reconfig_micros = reconfig.last_reconfiguration_time;
    let epoch_duration_micros = EPOCH_DURATION_SECS * 1_000_000;

    let elapsed_micros = now_micros.saturating_sub(last_reconfig_micros);
    let epochs_passed = elapsed_micros / epoch_duration_micros;
    let next_epoch_micros = last_reconfig_micros + ((epochs_passed + 1) * epoch_duration_micros);

    let next_epoch_secs = (next_epoch_micros / 1_000_000) as i64;
    DateTime::from_timestamp(next_epoch_secs, 0)
}

fn calculate_withdrawal_completion_date(reconfig: &ReconfigurationState, staking_config: &StakingConfig) -> Option<DateTime<Utc>> {
    let now_micros = Utc::now().timestamp_micros() as u64;
    let last_reconfig_micros = reconfig.last_reconfiguration_time;
    let lockup_duration_micros = staking_config.recurring_lockup_duration_secs * 1_000_000;

    let elapsed_micros = now_micros.saturating_sub(last_reconfig_micros);
    let periods_passed = elapsed_micros / lockup_duration_micros;
    let next_unlock_micros = last_reconfig_micros + ((periods_passed + 1) * lockup_duration_micros);

    let next_unlock_secs = (next_unlock_micros / 1_000_000) as i64;
    DateTime::from_timestamp(next_unlock_secs, 0)
}

pub fn map_delegations(stakes: Vec<(String, DelegationPoolStake)>, reconfig: &ReconfigurationState, staking_config: &StakingConfig) -> Vec<DelegationBase> {
    let asset_id = Chain::Aptos.as_asset_id();
    let next_epoch = calculate_pending_completion_date(reconfig);
    let withdrawal_completion = calculate_withdrawal_completion_date(reconfig, staking_config);

    stakes
        .into_iter()
        .flat_map(|(pool_address, stake)| {
            let mut delegations = Vec::new();

            if stake.active > BigUint::from(0u32) {
                delegations.push(map_delegation(&asset_id, DelegationState::Active, stake.active, &pool_address, None));
            }

            if stake.pending_active > BigUint::from(0u32) {
                delegations.push(map_delegation(
                    &asset_id,
                    DelegationState::Activating,
                    stake.pending_active,
                    &pool_address,
                    next_epoch,
                ));
            }

            if stake.pending_inactive > BigUint::from(0u32) {
                delegations.push(map_delegation(
                    &asset_id,
                    DelegationState::Deactivating,
                    stake.pending_inactive,
                    &pool_address,
                    withdrawal_completion,
                ));
            }

            if stake.inactive > BigUint::from(0u32) {
                delegations.push(map_delegation(
                    &asset_id,
                    DelegationState::Deactivating,
                    stake.inactive,
                    &pool_address,
                    withdrawal_completion,
                ));
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

    fn mock_stake(active: u32, inactive: u32, pending_active: u32, pending_inactive: u32) -> DelegationPoolStake {
        DelegationPoolStake {
            active: BigUint::from(active),
            inactive: BigUint::from(inactive),
            pending_active: BigUint::from(pending_active),
            pending_inactive: BigUint::from(pending_inactive),
        }
    }

    fn mock_reconfig(last_reconfiguration_time: u64) -> ReconfigurationState {
        ReconfigurationState {
            epoch: 100,
            last_reconfiguration_time,
        }
    }

    fn mock_config() -> StakingConfig {
        StakingConfig {
            rewards_rate: 0,
            rewards_rate_denominator: 1,
            recurring_lockup_duration_secs: 1209600,
        }
    }

    #[test]
    fn test_calculate_apy() {
        let config = StakingConfig {
            rewards_rate: 1600000000000,
            rewards_rate_denominator: 100000000000000000,
            recurring_lockup_duration_secs: 1209600,
        };

        assert!((calculate_apy(&config) - 7.0128).abs() < 0.01);
    }

    #[test]
    fn test_calculate_apy_zero_denominator() {
        let config = StakingConfig {
            rewards_rate: 1600000000000000,
            rewards_rate_denominator: 0,
            recurring_lockup_duration_secs: 1209600,
        };

        assert_eq!(calculate_apy(&config), 0.0);
    }

    #[test]
    fn test_map_delegations_active() {
        let delegations = map_delegations(vec![("pool".to_string(), mock_stake(1000, 0, 0, 0))], &mock_reconfig(1000000), &mock_config());

        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0].state, DelegationState::Active);
        assert_eq!(delegations[0].balance, BigUint::from(1000u32));
        assert!(delegations[0].completion_date.is_none());
    }

    #[test]
    fn test_map_delegations_pending_active() {
        let delegations = map_delegations(vec![("pool".to_string(), mock_stake(0, 0, 500, 0))], &mock_reconfig(1000000), &mock_config());

        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0].state, DelegationState::Activating);
        assert_eq!(delegations[0].balance, BigUint::from(500u32));
        assert!(delegations[0].completion_date.is_some());
    }

    #[test]
    fn test_map_delegations_pending_inactive() {
        let delegations = map_delegations(vec![("pool".to_string(), mock_stake(0, 0, 0, 300))], &mock_reconfig(1000000), &mock_config());

        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0].state, DelegationState::Deactivating);
        assert_eq!(delegations[0].balance, BigUint::from(300u32));
        assert!(delegations[0].completion_date.is_some());
    }

    #[test]
    fn test_map_delegations_inactive() {
        let now_micros = Utc::now().timestamp_micros() as u64;
        let recent_reconfig_micros = now_micros - (100_000 * 1_000_000);

        let delegations = map_delegations(
            vec![("pool".to_string(), mock_stake(0, 200, 0, 0))],
            &mock_reconfig(recent_reconfig_micros),
            &mock_config(),
        );

        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations[0].state, DelegationState::Deactivating);
        assert_eq!(delegations[0].balance, BigUint::from(200u32));
        assert!(delegations[0].completion_date.is_some());
    }

    #[test]
    fn test_map_delegations_multiple_states() {
        let delegations = map_delegations(
            vec![("pool".to_string(), mock_stake(1000, 200, 500, 300))],
            &mock_reconfig(1000000),
            &mock_config(),
        );

        assert_eq!(delegations.len(), 4);
    }
}

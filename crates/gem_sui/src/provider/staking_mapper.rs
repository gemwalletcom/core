use crate::models::RpcSuiSystemState;
use crate::models::staking::{SuiStakeDelegation, SuiSystemState, SuiValidators};
use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator, StakeValidator};

pub fn map_validators(validators: SuiValidators, default_apy: f64) -> Vec<DelegationValidator> {
    validators
        .apys
        .into_iter()
        .map(|validator| DelegationValidator {
            chain: Chain::Sui,
            id: validator.address,
            name: String::new(),
            is_active: true,
            commission: 0.0,
            apr: default_apy,
        })
        .collect()
}

pub fn map_delegations(delegations: Vec<SuiStakeDelegation>, system_state: SuiSystemState) -> Vec<DelegationBase> {
    let epoch_start_ms = system_state.epoch_start_timestamp_ms.parse::<i64>().unwrap_or(0);
    let epoch_duration_ms = system_state.epoch_duration_ms.parse::<i64>().unwrap_or(0);

    delegations
        .into_iter()
        .flat_map(|delegation| {
            let validator_address = delegation.validator_address.clone();
            delegation.stakes.into_iter().map(move |stake| {
                let completion_date = match map_stake_state(&stake.status) {
                    DelegationState::Activating => Some(DateTime::from_timestamp((epoch_start_ms + epoch_duration_ms) / 1000, 0).unwrap_or_else(Utc::now)),
                    _ => None,
                };

                DelegationBase {
                    asset_id: Chain::Sui.as_asset_id(),
                    state: map_stake_state(&stake.status),
                    balance: stake.principal,
                    shares: BigUint::from(0u32),
                    rewards: stake.estimated_reward.unwrap_or(BigUint::from(0u32)),
                    completion_date,
                    delegation_id: stake.staked_sui_id.clone(),
                    validator_id: validator_address.clone(),
                }
            })
        })
        .collect()
}

pub fn map_system_validators(system_state: RpcSuiSystemState) -> Vec<StakeValidator> {
    system_state
        .active_validators
        .into_iter()
        .map(|v| StakeValidator::new(v.sui_address, v.name))
        .collect()
}

pub fn map_staking_apy(validators: SuiValidators) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let max_apy = validators.apys.into_iter().map(|v| v.apy).fold(0.0, f64::max);
    Ok(max_apy * 100.0)
}

fn map_stake_state(status: &str) -> DelegationState {
    match status {
        "Active" => DelegationState::Active,
        "Pending" => DelegationState::Activating,
        _ => DelegationState::Pending,
    }
}

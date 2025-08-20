use primitives::{Chain, DelegationBase, DelegationValidator, DelegationState};
use chrono::{DateTime, Utc};
use crate::models::staking::{SuiStakeDelegation, SuiSystemState, SuiValidators};

pub fn map_validators(validators: SuiValidators, default_apy: f64) -> Vec<DelegationValidator> {
    validators.apys
        .into_iter()
        .map(|validator| {
            DelegationValidator {
                chain: Chain::Sui,
                id: validator.address,
                name: String::new(),
                is_active: true,
                commision: 0.0,
                apr: default_apy,
            }
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
                    DelegationState::Activating => {
                        Some(DateTime::from_timestamp((epoch_start_ms + epoch_duration_ms) / 1000, 0).unwrap_or_else(Utc::now))
                    },
                    _ => None,
                };

                DelegationBase {
                    asset_id: Chain::Sui.as_asset_id(),
                    state: map_stake_state(&stake.status),
                    balance: stake.principal.to_string(),
                    shares: "0".to_string(),
                    rewards: stake.estimated_reward.map(|r| r.to_string()).unwrap_or_else(|| "0".to_string()),
                    completion_date,
                    delegation_id: stake.staked_sui_id.clone(),
                    validator_id: validator_address.clone(),
                }
            })
        })
        .collect()
}

fn map_stake_state(status: &str) -> DelegationState {
    match status {
        "Active" => DelegationState::Active,
        "Pending" => DelegationState::Activating,
        _ => DelegationState::Pending,
    }
}
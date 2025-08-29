use crate::models::WitnessesList;
use primitives::{StakeValidator, DelegationValidator, Chain};

pub fn map_validators(witnesses: WitnessesList) -> Vec<StakeValidator> {
    witnesses.witnesses.into_iter().map(|x| StakeValidator::new(x.address, x.url)).collect()
}

pub fn map_staking_validators(witnesses: WitnessesList, apy: Option<f64>) -> Vec<DelegationValidator> {
    let default_apy = apy.unwrap_or(0.0);
    witnesses
        .witnesses
        .into_iter()
        .map(|witness| DelegationValidator {
            chain: Chain::Tron,
            id: witness.address,
            name: witness.url,
            is_active: true,
            commision: 0.0,
            apr: default_apy,
        })
        .collect()
}

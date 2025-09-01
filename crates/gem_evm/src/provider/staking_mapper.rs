use primitives::{DelegationBase, DelegationValidator};

pub fn map_staking_apy(_validators_data: String) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    unimplemented!("map_staking_apy")
}

pub fn map_validators(_validators_data: String, _default_apy: f64) -> Vec<DelegationValidator> {
    unimplemented!("map_validators")
}

pub fn map_delegations(_delegations_data: String, _system_state: String) -> Vec<DelegationBase> {
    unimplemented!("map_delegations")
}

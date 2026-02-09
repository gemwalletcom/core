use primitives::{EarnPositionData, EarnProvider};

pub fn map_staking_apy(_validators_data: String) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    unimplemented!("map_staking_apy")
}

pub fn map_validators(_validators_data: String, _default_apy: f64) -> Vec<EarnProvider> {
    unimplemented!("map_validators")
}

pub fn map_delegations(_delegations_data: String, _system_state: String) -> Vec<EarnPositionData> {
    unimplemented!("map_delegations")
}

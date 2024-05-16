use gem_bsc::stake_hub;

#[derive(uniffi::Enum, Debug)]
pub enum BscDelegationStatus {
    Active,
    Undelegating,
}

#[derive(uniffi::Record, Debug)]
pub struct BscDelegation {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: String,
    pub shares: String,
    pub status: BscDelegationStatus,
    pub unlock_time: Option<u64>,
}

#[derive(uniffi::Record, Debug)]
pub struct BscValidator {
    pub operator_address: String,
    pub moniker: String,
    pub commission: u64,
    pub apy: u64,
    pub jailed: bool,
}

impl From<stake_hub::BscDelegation> for BscDelegation {
    fn from(value: stake_hub::BscDelegation) -> Self {
        Self {
            delegator_address: value.delegator_address,
            validator_address: value.validator_address,
            amount: value.amount,
            shares: value.shares,
            status: BscDelegationStatus::Active,
            unlock_time: None,
        }
    }
}

impl From<stake_hub::BscUndelegation> for BscDelegation {
    fn from(value: stake_hub::BscUndelegation) -> Self {
        Self {
            delegator_address: value.delegator_address,
            validator_address: value.validator_address,
            amount: value.amount,
            shares: value.shares,
            status: BscDelegationStatus::Undelegating,
            unlock_time: value.unlock_time.parse::<u64>().ok(),
        }
    }
}

impl From<stake_hub::BscValidator> for BscValidator {
    fn from(value: stake_hub::BscValidator) -> Self {
        Self {
            operator_address: value.operator_address,
            moniker: value.moniker,
            commission: value.commission,
            apy: value.apy,
            jailed: value.jailed,
        }
    }
}

pub fn decode_delegations_return(result: &[u8]) -> Result<Vec<BscDelegation>, anyhow::Error> {
    stake_hub::decode_delegations_return(result)
        .map(|value| value.into_iter().map(BscDelegation::from).collect())
}

pub fn decode_undelegations_return(result: &[u8]) -> Result<Vec<BscDelegation>, anyhow::Error> {
    stake_hub::decode_undelegations_return(result)
        .map(|value| value.into_iter().map(BscDelegation::from).collect())
}

pub fn decode_validators_return(result: &[u8]) -> Result<Vec<BscValidator>, anyhow::Error> {
    stake_hub::decode_validators_return(result)
        .map(|value| value.into_iter().map(BscValidator::from).collect())
}

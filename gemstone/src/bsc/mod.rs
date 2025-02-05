use crate::GemstoneError;
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

/// Exports functions
#[uniffi::export]
pub fn bsc_encode_validators_call(offset: u16, limit: u16) -> Vec<u8> {
    stake_hub::encode_validators_call(offset, limit)
}

#[uniffi::export]
pub fn bsc_decode_validators_return(result: Vec<u8>) -> Result<Vec<BscValidator>, GemstoneError> {
    stake_hub::decode_validators_return(&result)
        .map(|value| value.into_iter().map(BscValidator::from).collect())
        .map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_delegations_call(delegator: &str, offset: u16, limit: u16) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_delegations_call(delegator, offset, limit).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_decode_delegations_return(result: Vec<u8>) -> Result<Vec<BscDelegation>, GemstoneError> {
    stake_hub::decode_delegations_return(&result)
        .map(|value| value.into_iter().map(BscDelegation::from).collect())
        .map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_undelegations_call(delegator: &str, offset: u16, limit: u16) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_undelegations_call(delegator, offset, limit).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_decode_undelegations_return(result: Vec<u8>) -> Result<Vec<BscDelegation>, GemstoneError> {
    stake_hub::decode_undelegations_return(&result)
        .map(|value| value.into_iter().map(BscDelegation::from).collect())
        .map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_delegate_call(operator_address: String, delegate_vote_power: bool) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_delegate_call(&operator_address, delegate_vote_power).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_undelegate_call(operator_address: String, shares: String) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_undelegate_call(&operator_address, &shares).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_redelegate_call(src_validator: String, dst_validator: String, shares: String, delegate_vote_power: bool) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_redelegate_call(&src_validator, &dst_validator, &shares, delegate_vote_power).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_claim_call(operator_address: String, request_number: u64) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_claim_call(&operator_address, request_number).map_err(GemstoneError::from)
}

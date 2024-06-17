use crate::GemstoneError;
use gem_evm::{erc2612::Permit, lido, lido::WithdrawalRequestStatus};

#[derive(uniffi::Record, Debug)]
pub struct ERC2612Permit {
    pub value: String,
    pub deadline: u64,
    pub signature: Vec<u8>,
}

impl From<ERC2612Permit> for Permit {
    fn from(permit: ERC2612Permit) -> Self {
        Permit {
            value: permit.value,
            deadline: permit.deadline.to_string(),
            v: permit.signature[64],
            r: permit.signature[0..32].to_vec(),
            s: permit.signature[32..64].to_vec(),
        }
    }
}

#[derive(uniffi::Record, Debug)]
pub struct LidoWithdrawalRequest {
    pub amount: String,
    pub shares: String,
    pub owner: String,
    pub timestamp: u64,
    pub is_finalized: bool,
    pub is_claimed: bool,
}

impl From<WithdrawalRequestStatus> for LidoWithdrawalRequest {
    fn from(value: WithdrawalRequestStatus) -> Self {
        Self {
            amount: value.amountOfStETH.to_string(),
            shares: value.amountOfShares.to_string(),
            owner: value.owner.to_string(),
            timestamp: value.timestamp.to_string().parse::<u64>().unwrap(),
            is_finalized: value.isFinalized,
            is_claimed: value.isClaimed,
        }
    }
}

#[uniffi::export]
pub fn lido_encode_submit(referral: String) -> Result<Vec<u8>, GemstoneError> {
    lido::encode_submit_with_referral(&referral).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn lido_encode_request_withdrawals(
    amounts: Vec<String>,
    owner: String,
    permit: ERC2612Permit,
) -> Result<Vec<u8>, GemstoneError> {
    lido::encode_request_withdrawals_with_permit(amounts, &owner, &permit.into())
        .map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn lido_decode_request_withdrawals_return(
    result: Vec<u8>,
) -> Result<Vec<String>, GemstoneError> {
    lido::decode_request_withdrawals_return(&result).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn lido_encode_claim_withdrawal(request_id: String) -> Result<Vec<u8>, GemstoneError> {
    lido::encode_claim_withdrawal(&request_id).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn lido_encode_withdrawal_request_ids(owner: String) -> Result<Vec<u8>, GemstoneError> {
    lido::encode_get_withdrawal_request_ids(&owner).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn lido_decode_withdrawal_request_ids(result: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
    lido::decode_get_withdrawal_request_ids(&result).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn lido_encode_withdrawal_statuses(request_ids: Vec<String>) -> Result<Vec<u8>, GemstoneError> {
    lido::encode_get_withdrawal_request_status(&request_ids).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn lido_decode_get_withdrawal_statuses(
    result: Vec<u8>,
) -> Result<Vec<LidoWithdrawalRequest>, GemstoneError> {
    lido::decode_get_withdrawal_request_status(&result)
        .map(|x| x.into_iter().map(LidoWithdrawalRequest::from).collect())
        .map_err(GemstoneError::from)
}

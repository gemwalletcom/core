use anyhow::Error;
use gem_evm::{erc2612::Permit, lido, lido::WithdrawalRequestStatus};

#[derive(uniffi::Record, Debug)]
pub struct ERC2612Permit {
    pub value: String,
    pub deadline: String,
    pub signature: Vec<u8>,
}

#[derive(uniffi::Record, Debug)]
pub struct LidoWithdrawalRequest {
    pub amount: String,
    pub shares: String,
    pub owner: String,
    pub timestamp: String,
    pub is_finalized: bool,
    pub is_claimed: bool,
}

impl From<WithdrawalRequestStatus> for LidoWithdrawalRequest {
    fn from(value: WithdrawalRequestStatus) -> Self {
        Self {
            amount: value.amountOfStETH.to_string(),
            shares: value.amountOfShares.to_string(),
            owner: value.owner.to_string(),
            timestamp: value.timestamp.to_string(),
            is_finalized: value.isFinalized,
            is_claimed: value.isClaimed,
        }
    }
}

pub fn encode_submit_with_referral(referral: &str) -> Result<Vec<u8>, Error> {
    lido::encode_submit_with_referral(referral)
}

pub fn encode_request_withdrawals_with_permit(
    amounts: Vec<String>,
    owner: String,
    permit: ERC2612Permit,
) -> Result<Vec<u8>, Error> {
    lido::encode_request_withdrawals_with_permit(amounts, &owner, &permit.into())
}

pub fn encode_claim_withdrawal(request_id: &str) -> Result<Vec<u8>, Error> {
    lido::encode_claim_withdrawal(request_id)
}

pub fn decode_request_withdrawals_return(result: &[u8]) -> Result<Vec<String>, Error> {
    lido::decode_request_withdrawals_return(result)
}

pub fn encode_get_withdrawal_request_ids(owner: &str) -> Result<Vec<u8>, Error> {
    lido::encode_get_withdrawal_request_ids(owner)
}

pub fn decode_get_withdrawal_request_ids(result: &[u8]) -> Result<Vec<String>, Error> {
    lido::decode_get_withdrawal_request_ids(result)
}

pub fn encode_get_withdrawal_request_status(request_ids: &[String]) -> Result<Vec<u8>, Error> {
    lido::encode_get_withdrawal_request_status(request_ids)
}

pub fn decode_get_withdrawal_request_status(
    result: &[u8],
) -> Result<Vec<LidoWithdrawalRequest>, Error> {
    lido::decode_get_withdrawal_request_status(result)
        .map(|x| x.into_iter().map(LidoWithdrawalRequest::from).collect())
}

impl From<ERC2612Permit> for Permit {
    fn from(permit: ERC2612Permit) -> Self {
        Permit {
            value: permit.value,
            deadline: permit.deadline,
            v: permit.signature[64],
            r: permit.signature[0..32].to_vec(),
            s: permit.signature[32..64].to_vec(),
        }
    }
}

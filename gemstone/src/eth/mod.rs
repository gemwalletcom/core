use anyhow::Error;
use gem_evm::{erc2612::Permit, lido};

#[derive(uniffi::Record, Debug)]
pub struct ERC2612Permit {
    pub value: String,
    pub deadline: String,
    pub signature: Vec<u8>,
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

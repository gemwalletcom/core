use primitives::{SignerError, SignerInput};

#[derive(Clone, Copy)]
pub struct TransactionParams {
    pub nonce: u64,
    pub chain_id: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub gas_limit: u64,
}

impl TransactionParams {
    pub fn from_input(input: &SignerInput) -> Result<Self, SignerError> {
        Ok(Self {
            nonce: input.metadata.get_sequence()?,
            chain_id: input.metadata.get_chain_id_u64()?,
            max_fee_per_gas: input.fee.gas_price_u64()? as u128,
            max_priority_fee_per_gas: input.fee.priority_fee_u64()? as u128,
            gas_limit: input.fee.gas_limit()?,
        })
    }
}

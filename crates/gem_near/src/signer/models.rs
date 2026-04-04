use super::serialization::parse_u128_le;
use primitives::{SignerError, SignerInput};

pub struct NearTransfer {
    pub signer_id: String,
    pub receiver_id: String,
    pub nonce: u64,
    pub block_hash: Vec<u8>,
    pub deposit: [u8; 16],
}

impl NearTransfer {
    pub fn from_input(input: &SignerInput) -> Result<Self, SignerError> {
        let block_hash = bs58::decode(input.metadata.get_block_hash().map_err(SignerError::from_display)?)
            .into_vec()
            .map_err(|e| SignerError::invalid_input(format!("invalid Near block hash: {e}")))?;

        Ok(Self {
            signer_id: input.sender_address.clone(),
            receiver_id: input.destination_address.clone(),
            nonce: input.metadata.get_sequence().map_err(SignerError::from_display)?,
            block_hash,
            deposit: parse_u128_le(&input.value)?,
        })
    }
}

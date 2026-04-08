use primitives::{SignerError, SignerInput};

pub struct NearTransfer {
    pub signer_id: String,
    pub receiver_id: String,
    pub nonce: u64,
    pub block_hash: [u8; 32],
    pub deposit: [u8; 16],
}

impl NearTransfer {
    pub fn from_input(input: &SignerInput) -> Result<Self, SignerError> {
        let block_hash: [u8; 32] = bs58::decode(input.metadata.get_block_hash().map_err(SignerError::from_display)?)
            .into_vec()
            .map_err(|e| SignerError::invalid_input(format!("invalid NEAR block hash: {e}")))?
            .try_into()
            .map_err(|_| SignerError::invalid_input("NEAR block hash must be 32 bytes"))?;

        Ok(Self {
            signer_id: input.sender_address.clone(),
            receiver_id: input.destination_address.clone(),
            nonce: input.metadata.get_sequence().map_err(SignerError::from_display)?,
            block_hash,
            deposit: input.value.parse::<u128>().map_err(|_| SignerError::invalid_input("invalid NEAR amount"))?.to_le_bytes(),
        })
    }
}

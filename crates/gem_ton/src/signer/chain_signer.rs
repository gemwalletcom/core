use std::time::{SystemTime, UNIX_EPOCH};

use primitives::{ChainSigner, SignerError, SignerInput};

use super::signature::sign_personal;
use super::transaction;

#[derive(Default)]
pub struct TonChainSigner;

impl ChainSigner for TonChainSigner {
    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SignerError::InvalidInput(e.to_string()))?
            .as_secs();
        let result = sign_personal(message, private_key, timestamp)?;
        Ok(gem_encoding::encode_base64(&result.signature))
    }

    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        transaction::sign_transfer(input, private_key, None)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        transaction::sign_token_transfer(input, private_key, None)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        transaction::sign_swap(input, private_key, None)
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        transaction::sign_data(input, private_key, None)
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

use primitives::{ChainSigner, SignerError, SignerInput};

use super::signer::TonSigner;

#[derive(Default)]
pub struct TonChainSigner;

impl ChainSigner for TonChainSigner {
    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SignerError::InvalidInput(e.to_string()))?
            .as_secs();
        let result = TonSigner::new(private_key)?.sign_personal(message, timestamp)?;
        Ok(gem_encoding::encode_base64(&result.signature))
    }

    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        TonSigner::new(private_key)?.sign_transfer(input, None)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        TonSigner::new(private_key)?.sign_token_transfer(input, None)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        TonSigner::new(private_key)?.sign_swap(input, None)
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        TonSigner::new(private_key)?.sign_data(input, None)
    }
}

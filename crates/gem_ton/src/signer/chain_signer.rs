use std::time::{SystemTime, UNIX_EPOCH};

use primitives::{ChainSigner, SignerError, SignerInput};

use super::signature::sign_personal;

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
        self.sign_from_metadata(input, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_from_metadata(input, private_key)
    }
}

impl TonChainSigner {
    fn sign_from_metadata(&self, _input: &SignerInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::signing_error("TON transaction signing not yet implemented in chain signer"))
    }
}

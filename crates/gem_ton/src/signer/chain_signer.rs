use std::time::{SystemTime, UNIX_EPOCH};

use primitives::{ChainSigner, SignerError, TransactionLoadInput};

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
        Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result.signature))
    }

    fn sign_transfer(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_from_metadata(input, private_key)
    }

    fn sign_token_transfer(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_from_metadata(input, private_key)
    }
}

impl TonChainSigner {
    fn sign_from_metadata(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        todo!("TON transaction signing not yet implemented in chain signer")
    }
}

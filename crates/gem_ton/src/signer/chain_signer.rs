use primitives::{ChainSigner, SignerError, TransactionLoadInput};

use super::signature::sign_personal;

#[derive(Default)]
pub struct TonChainSigner;

impl ChainSigner for TonChainSigner {
    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let (signature, _public_key) = sign_personal(message, private_key)?;
        Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature))
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

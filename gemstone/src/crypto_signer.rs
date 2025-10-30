use crate::GemstoneError;
use ::signer::{SignatureScheme as GemSignatureScheme, Signer};

#[derive(Default, uniffi::Object)]
pub struct CryptoSigner;

#[uniffi::export]
impl CryptoSigner {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn sign_sui_personal_message(&self, message: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        Signer::sign_sui_personal_message(message, private_key).map_err(GemstoneError::from)
    }

    pub fn sign_sui_digest(&self, digest: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        Signer::sign_sui_digest(digest, private_key).map_err(GemstoneError::from)
    }

    pub fn sign_digest(&self, scheme: GemSignatureScheme, digest: Vec<u8>, private_key: Vec<u8>) -> Result<Vec<u8>, GemstoneError> {
        Signer::sign_digest(scheme, digest, private_key).map_err(GemstoneError::from)
    }
}

#[uniffi::remote(Enum)]
pub enum GemSignatureScheme {
    Ed25519,
    Secp256k1,
}

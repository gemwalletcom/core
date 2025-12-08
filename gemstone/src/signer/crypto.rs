use gem_ton::address::base64_to_hex_address;
use signer::{SignatureScheme as GemSignatureScheme, Signer, TonSignDataInput as SignerTonSignDataInput};

use crate::GemstoneError;

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

    pub fn sign_ton_personal_message(&self, domain: String, payload: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let signer_input = SignerTonSignDataInput { domain, payload };
        Signer::sign_ton_personal_message(signer_input, private_key).map_err(GemstoneError::from)
    }

    pub fn ton_base64_to_raw_address(&self, base64_address: String) -> Result<String, GemstoneError> {
        base64_to_hex_address(base64_address).map_err(|e| GemstoneError::AnyError { msg: e.to_string() })
    }
}

#[uniffi::remote(Enum)]
pub enum GemSignatureScheme {
    Ed25519,
    Secp256k1,
}

mod ed25519;
mod eip712;
mod error;
mod secp256k1;

use zeroize::Zeroizing;

use crate::ed25519::sign_digest as sign_ed25519_digest;
use crate::secp256k1::sign_digest as sign_secp256k1_digest;

pub use ed25519::signing_key_from_bytes;
pub use eip712::hash_typed_data as hash_eip712;
pub use error::SignerError;

#[derive(Debug, Default)]
pub struct Signer;

#[derive(Clone, Copy, Debug)]
pub enum SignatureScheme {
    Ed25519,
    Secp256k1,
}

impl Signer {
    pub fn sign_digest(scheme: SignatureScheme, digest: Vec<u8>, private_key: Vec<u8>) -> Result<Vec<u8>, SignerError> {
        let private_key = Zeroizing::new(private_key);
        match scheme {
            SignatureScheme::Ed25519 => Ok(sign_ed25519_digest(&digest, &private_key)?.to_bytes().to_vec()),
            SignatureScheme::Secp256k1 => sign_secp256k1_digest(&digest, &private_key),
        }
    }

    pub fn sign_eip712(typed_data_json: &str, private_key: &[u8]) -> Result<String, SignerError> {
        let digest = eip712::hash_typed_data(typed_data_json)?;
        let private_key_vec = Zeroizing::new(private_key.to_vec());
        let signature = Self::sign_digest(SignatureScheme::Secp256k1, digest.to_vec(), private_key_vec.to_vec())?;
        Ok(hex::encode(signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ed25519_signing_key_rejects_invalid_length() {
        let result = signing_key_from_bytes(&[0u8; 16]);
        assert!(result.is_err());
    }
}

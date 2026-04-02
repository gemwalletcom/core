mod decode;
mod ed25519;
mod eip712;
mod secp256k1;

use ed25519_dalek::Signer as DalekSigner;
use zeroize::Zeroizing;

use crate::ed25519::{sign_digest as sign_ed25519_digest, signing_key_from_bytes};
pub use crate::secp256k1::public_key_from_private as secp256k1_public_key;
use crate::secp256k1::{sign_digest as sign_secp256k1_digest, sign_ethereum_digest as sign_secp256k1_ethereum_digest};

pub use decode::{decode_private_key, encode_private_key, supports_private_key_import};
pub use eip712::hash_typed_data as hash_eip712;
pub use primitives::SignerError;

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

    /// Sign a secp256k1 digest with Ethereum-style recovery id.
    /// Returns [r(32), s(32), v(1)] where v ∈ {27, 28}.
    pub fn sign_secp256k1_ethereum(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
        let private_key = Zeroizing::new(private_key.to_vec());
        sign_secp256k1_ethereum_digest(digest, &private_key)
    }

    /// Sign a digest with Ed25519 and return both the signature and public key bytes.
    /// Returns (signature_bytes, public_key_bytes) where signature is 64 bytes and public key is 32 bytes.
    pub fn sign_ed25519_with_public_key(digest: &[u8], private_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), SignerError> {
        let private_key = Zeroizing::new(private_key.to_vec());
        let signing_key = signing_key_from_bytes(&private_key)?;
        let signature = signing_key.sign(digest);
        let signature_bytes = signature.to_bytes().to_vec();
        let public_key_bytes = signing_key.verifying_key().to_bytes().to_vec();
        Ok((signature_bytes, public_key_bytes))
    }

    pub fn sign_eip712(typed_data_json: &str, private_key: &[u8]) -> Result<String, SignerError> {
        let digest = eip712::hash_typed_data(typed_data_json)?;
        let signature = Self::sign_secp256k1_ethereum(&digest, private_key)?;
        Ok(hex::encode(signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIVATE_KEY: &str = "1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34";

    #[test]
    fn ed25519_sign_with_public_key_rejects_invalid_length() {
        let result = Signer::sign_ed25519_with_public_key(&[0u8; 32], &[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn ed25519_sign_with_public_key_returns_correct_lengths() {
        let private_key = hex::decode(PRIVATE_KEY).unwrap();
        let digest = b"test message";

        let (signature, public_key) = Signer::sign_ed25519_with_public_key(digest, &private_key).unwrap();

        assert_eq!(signature.len(), 64, "Ed25519 signature should be 64 bytes");
        assert_eq!(public_key.len(), 32, "Ed25519 public key should be 32 bytes");
    }

    #[test]
    fn sign_secp256k1_ethereum_returns_ethereum_recovery_id() {
        let private_key = hex::decode(PRIVATE_KEY).unwrap();
        let digest = [7u8; 32];

        let signature = Signer::sign_secp256k1_ethereum(&digest, &private_key).unwrap();

        assert_eq!(signature.len(), 65);
        assert!(matches!(signature[64], 27 | 28), "v must be 27 or 28, got {}", signature[64]);
    }

    #[test]
    fn sign_secp256k1_ethereum_rejects_invalid_key() {
        let result = Signer::sign_secp256k1_ethereum(&[7u8; 32], &[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn sign_eip712_returns_ethereum_recovery_id() {
        let private_key = hex::decode(PRIVATE_KEY).unwrap();
        // Minimal valid EIP-712 typed data
        let typed_data = r#"{
            "types": {
                "EIP712Domain": [{"name":"name","type":"string"}],
                "Message": [{"name":"content","type":"string"}]
            },
            "primaryType": "Message",
            "domain": {"name": "Test"},
            "message": {"content": "Hello"}
        }"#;

        let hex_sig = Signer::sign_eip712(typed_data, &private_key).unwrap();
        let sig_bytes = hex::decode(&hex_sig).unwrap();

        assert_eq!(sig_bytes.len(), 65);
        assert!(matches!(sig_bytes[64], 27 | 28), "EIP-712 v must be 27 or 28, got {}", sig_bytes[64]);
    }
}

mod base32;
mod decode;
mod ed25519;
mod eip712;
mod secp256k1;

#[cfg(test)]
pub(crate) mod testkit {
    pub const TEST_PRIVATE_KEY: &str = "1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34";
}

use zeroize::Zeroizing;

use crate::ed25519::sign_digest as sign_ed25519_digest;
pub use crate::secp256k1::{RECOVERY_ID_INDEX, SIGNATURE_LENGTH, apply_eth_recovery_id, public_key_from_private as secp256k1_public_key};

pub use base32::{Base32Address, decode_base32};
pub use decode::{decode_private_key, encode_private_key, supports_private_key_import};
pub use ed25519::Ed25519KeyPair;
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
            SignatureScheme::Secp256k1 => secp256k1::sign_digest_append_recovery(&digest, &private_key),
        }
    }

    /// Sign a secp256k1 digest returning [r(32), s(32), v(1)] where v ∈ {27, 28}.
    pub fn sign_eth_digest(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, SignerError> {
        let private_key = Zeroizing::new(private_key.to_vec());
        secp256k1::sign_eth_digest(digest, &private_key)
    }

    pub fn sign_eip712(typed_data_json: &str, private_key: &[u8]) -> Result<String, SignerError> {
        let digest = eip712::hash_typed_data(typed_data_json)?;
        let signature = Self::sign_eth_digest(&digest, private_key)?;
        Ok(hex::encode(signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::TEST_PRIVATE_KEY;

    #[test]
    fn ed25519_key_pair_rejects_invalid_length() {
        let result = Ed25519KeyPair::from_private_key(&[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn ed25519_key_pair_returns_correct_lengths() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let digest = b"test message";
        let key_pair = Ed25519KeyPair::from_private_key(&private_key).unwrap();

        let signature = key_pair.sign(digest);
        let public_key = key_pair.public_key_bytes.to_vec();

        assert_eq!(signature.len(), 64, "Ed25519 signature should be 64 bytes");
        assert_eq!(public_key.len(), 32, "Ed25519 public key should be 32 bytes");
    }

    #[test]
    fn ed25519_key_pair_public_key_is_stable() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let first = Ed25519KeyPair::from_private_key(&private_key).unwrap();
        let second = Ed25519KeyPair::from_private_key(&private_key).unwrap();

        assert_eq!(first.public_key_bytes, second.public_key_bytes);
    }

    #[test]
    fn ed25519_key_pair_reuses_signing_key_for_public_key_and_signature() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let digest = b"test message";

        let key_pair = Ed25519KeyPair::from_private_key(&private_key).unwrap();
        let signature = key_pair.sign(digest);
        let public_key = key_pair.public_key_bytes.to_vec();

        let another_key_pair = Ed25519KeyPair::from_private_key(&private_key).unwrap();
        let expected_signature = another_key_pair.sign(digest);
        let expected_public_key = another_key_pair.public_key_bytes.to_vec();
        assert_eq!(signature, expected_signature);
        assert_eq!(public_key, expected_public_key);
    }
}

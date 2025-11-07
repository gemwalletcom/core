mod ed25519;
mod eip712;
mod error;
mod secp256k1;
mod sui;

pub use eip712::hash_typed_data as hash_eip712;
pub use error::SignerError;
pub use sui::SUI_PERSONAL_MESSAGE_SIGNATURE_LEN;

use crate::ed25519::{sign_digest as sign_ed25519_digest, signing_key_from_bytes};
use crate::secp256k1::sign_digest as sign_secp256k1_digest;
use crate::sui::assemble_signature;
use ed25519_dalek::Signer as DalekSigner;
use std::borrow::Cow;
use sui_types::PersonalMessage;
use zeroize::Zeroizing;

#[derive(Debug, Default)]
pub struct Signer;

#[derive(Clone, Copy, Debug)]
pub enum SignatureScheme {
    Ed25519,
    Secp256k1,
}

impl Signer {
    pub fn sign_sui_personal_message(message: Vec<u8>, private_key: Vec<u8>) -> Result<String, SignerError> {
        let private_key = Zeroizing::new(private_key);
        let personal_message = PersonalMessage(Cow::Owned(message));
        let digest = personal_message.signing_digest();
        Self::sign_sui_digest(digest.to_vec(), private_key.to_vec())
    }

    pub fn sign_sui_digest(digest: Vec<u8>, private_key: Vec<u8>) -> Result<String, SignerError> {
        let private_key = Zeroizing::new(private_key);
        let signing_key = signing_key_from_bytes(&private_key)?;
        let signature = signing_key.sign(digest.as_slice());
        let signature_bytes = signature.to_bytes();
        let public_key_bytes = signing_key.verifying_key().to_bytes();

        assemble_signature(&signature_bytes, &public_key_bytes)
    }

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
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;
    use ed25519_dalek::{Signature, SigningKey, Verifier};

    #[test]
    fn test_sui_sign_personal_message() {
        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").expect("valid hex");
        let message = b"Hello, world!".to_vec();

        let signature_base64 = Signer::sign_sui_personal_message(message.clone(), private_key.clone()).expect("signing succeeds");

        let signature_bytes = STANDARD.decode(&signature_base64).expect("valid base64");
        assert_eq!(signature_bytes.len(), SUI_PERSONAL_MESSAGE_SIGNATURE_LEN, "signature layout length");
        assert_eq!(signature_bytes[0], 0x00, "expected Ed25519 flag prefix");

        let signature = &signature_bytes[1..65];
        let public_key_bytes = &signature_bytes[65..];

        let key_bytes: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key.clone().try_into().expect("32 byte secret key");
        let signing_key = SigningKey::from_bytes(&key_bytes);
        assert_eq!(public_key_bytes, signing_key.verifying_key().as_bytes(), "public key suffix matches secret key");

        let personal_message = PersonalMessage(Cow::Borrowed(message.as_slice()));
        let digest = personal_message.signing_digest();
        let signature = Signature::from_bytes(signature.try_into().expect("64 byte signature"));

        signing_key
            .verifying_key()
            .verify(digest.as_ref(), &signature)
            .expect("signature verifies against digest");

        let expected_base64 =
            "ALmKZNcvdmYgYloqKMAq7eSw5neV1mSEKfZProHEh8Ddw+6aJvLpuViFqZCHqwKdCqtzN8a+7jIDQSxbvmt04QDTaUUhl8KlZIHl4tPovwPeI0n2emMVGVaCIgjCM0re4g==";
        assert_eq!(signature_base64, expected_base64);
    }

    #[test]
    fn ed25519_signing_key_rejects_invalid_length() {
        let result = signing_key_from_bytes(&[0u8; 16]);
        assert!(result.is_err());
    }
}

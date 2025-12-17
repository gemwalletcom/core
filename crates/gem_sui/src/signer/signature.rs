use std::borrow::Cow;

use primitives::SignerError;
use signer::Signer;
use sui_types::{Ed25519PublicKey, Ed25519Signature, PersonalMessage, SimpleSignature, UserSignature};

/// 1-byte flag + 64-byte signature + 32-byte public key.
pub const SUI_PERSONAL_MESSAGE_SIGNATURE_LEN: usize = 1 + Ed25519Signature::LENGTH + Ed25519PublicKey::LENGTH;

pub fn sign_personal_message(message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
    let personal_message = PersonalMessage(Cow::Borrowed(message));
    let digest = personal_message.signing_digest();
    sign_digest(digest.as_ref(), private_key)
}

pub fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
    let (signature_bytes, public_key_bytes) =
        Signer::sign_ed25519_with_public_key(digest, private_key).map_err(|e| SignerError::InvalidInput(e.to_string()))?;

    assemble_signature(&signature_bytes, &public_key_bytes)
}

fn assemble_signature(signature: &[u8], public_key: &[u8]) -> Result<String, SignerError> {
    let signature_bytes: [u8; Ed25519Signature::LENGTH] = signature
        .try_into()
        .map_err(|_| SignerError::InvalidInput(format!("Expected {} byte ed25519 signature, got {}", Ed25519Signature::LENGTH, signature.len())))?;
    let public_key_bytes: [u8; Ed25519PublicKey::LENGTH] = public_key.try_into().map_err(|_| {
        SignerError::InvalidInput(format!(
            "Expected {} byte ed25519 public key, got {}",
            Ed25519PublicKey::LENGTH,
            public_key.len()
        ))
    })?;

    let sui_signature = SimpleSignature::Ed25519 {
        signature: Ed25519Signature::new(signature_bytes),
        public_key: Ed25519PublicKey::new(public_key_bytes),
    };

    Ok(UserSignature::Simple(sui_signature).to_base64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;

    #[test]
    fn test_sui_sign_personal_message() {
        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").expect("valid hex");
        let message = b"Hello, world!".to_vec();

        let signature_base64 = sign_personal_message(&message, &private_key).expect("signing succeeds");

        let signature_bytes = STANDARD.decode(&signature_base64).expect("valid base64");
        assert_eq!(signature_bytes.len(), SUI_PERSONAL_MESSAGE_SIGNATURE_LEN, "signature layout length");
        assert_eq!(signature_bytes[0], 0x00, "expected Ed25519 flag prefix");

        let expected_base64 =
            "ALmKZNcvdmYgYloqKMAq7eSw5neV1mSEKfZProHEh8Ddw+6aJvLpuViFqZCHqwKdCqtzN8a+7jIDQSxbvmt04QDTaUUhl8KlZIHl4tPovwPeI0n2emMVGVaCIgjCM0re4g==";
        assert_eq!(signature_base64, expected_base64);
    }

    #[test]
    fn test_sign_digest_rejects_invalid_key_length() {
        let result = sign_digest(&[0u8; 32], &[0u8; 16]);
        assert!(result.is_err());
    }
}

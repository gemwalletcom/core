use crate::error::SignerError;
use sui_types::{Ed25519PublicKey, Ed25519Signature, SimpleSignature, UserSignature};

/// 1-byte flag + 64-byte signature + 32-byte public key.
pub const SUI_PERSONAL_MESSAGE_SIGNATURE_LEN: usize = 1 + Ed25519Signature::LENGTH + Ed25519PublicKey::LENGTH;

pub(crate) fn assemble_signature(signature: &[u8], public_key: &[u8]) -> Result<String, SignerError> {
    let signature_bytes: [u8; Ed25519Signature::LENGTH] = signature
        .try_into()
        .map_err(|_| SignerError::new(format!("Expected {} byte ed25519 signature, got {}", Ed25519Signature::LENGTH, signature.len())))?;
    let public_key_bytes: [u8; Ed25519PublicKey::LENGTH] = public_key.try_into().map_err(|_| {
        SignerError::new(format!(
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

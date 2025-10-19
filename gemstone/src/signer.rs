use crate::GemstoneError;
use ed25519_dalek::{Signer, SigningKey};
use k256::ecdsa::SigningKey as SecpSigningKey;
use std::borrow::Cow;
use sui_types::{Ed25519PublicKey, Ed25519Signature, PersonalMessage, SimpleSignature, UserSignature};

#[derive(Default, uniffi::Object)]
pub struct GemstoneSigner;

#[uniffi::export]
impl GemstoneSigner {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn sign_sui_personal_message(&self, message: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let personal_message = PersonalMessage(Cow::Owned(message));
        let digest = personal_message.signing_digest();
        self.sign_sui_digest(digest.to_vec(), private_key)
    }

    pub fn sign_sui_digest(&self, digest: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let signature = sign_ed25519(&digest, &private_key)?;
        assemble_sui_signature(signature.to_vec(), private_key)
    }

    pub fn sign_digest(&self, algorithm: GemSignatureAlgorithm, digest: Vec<u8>, private_key: Vec<u8>) -> Result<Vec<u8>, GemstoneError> {
        match algorithm {
            GemSignatureAlgorithm::Ed25519 => Ok(sign_ed25519(&digest, &private_key)?.to_bytes().to_vec()),
            GemSignatureAlgorithm::Secp256k1 => sign_secp256k1(&digest, &private_key),
        }
    }
}

fn sign_ed25519(digest: &[u8], private_key: &[u8]) -> Result<ed25519_dalek::Signature, GemstoneError> {
    let key_bytes: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key.try_into().map_err(|_| GemstoneError::from("Invalid Ed25519 private key length"))?;

    let signing_key = SigningKey::from_bytes(&key_bytes);
    Ok(signing_key.sign(digest))
}

fn assemble_sui_signature(signature: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let key_bytes: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key
        .as_slice()
        .try_into()
        .map_err(|_| GemstoneError::from("Invalid Ed25519 private key length"))?;

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let public_key = signing_key.verifying_key().to_bytes();

    let signature_bytes: [u8; Ed25519Signature::LENGTH] = signature
        .as_slice()
        .try_into()
        .map_err(|_| GemstoneError::from(format!("Expected {} byte ed25519 signature, got {}", Ed25519Signature::LENGTH, signature.len())))?;
    let public_key_bytes: [u8; Ed25519PublicKey::LENGTH] = public_key.as_slice().try_into().expect("derived Ed25519 public key must be 32 bytes");

    let sui_signature = SimpleSignature::Ed25519 {
        signature: Ed25519Signature::new(signature_bytes),
        public_key: Ed25519PublicKey::new(public_key_bytes),
    };

    Ok(UserSignature::Simple(sui_signature).to_base64())
}

fn sign_secp256k1(digest: &[u8], private_key: &[u8]) -> Result<Vec<u8>, GemstoneError> {
    let signing_key = SecpSigningKey::from_slice(private_key).map_err(|_| GemstoneError::from("Invalid Secp256k1 private key"))?;
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(digest)
        .map_err(|_| GemstoneError::from("Failed to sign Secp256k1 digest"))?;

    let mut out = signature.to_bytes().to_vec();
    out.push(u8::from(recovery_id));
    Ok(out)
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum GemSignatureAlgorithm {
    Ed25519,
    Secp256k1,
}

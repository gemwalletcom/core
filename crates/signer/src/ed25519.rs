use ed25519_dalek::{Signer as DalekSigner, SigningKey};

use crate::error::SignerError;

pub(crate) fn signing_key_from_bytes(private_key: &[u8]) -> Result<SigningKey, SignerError> {
    let key_bytes: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key.try_into().map_err(|_| SignerError::new("Invalid Ed25519 private key length"))?;
    Ok(SigningKey::from_bytes(&key_bytes))
}

pub(crate) fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<ed25519_dalek::Signature, SignerError> {
    let signing_key = signing_key_from_bytes(private_key)?;
    Ok(signing_key.sign(digest))
}

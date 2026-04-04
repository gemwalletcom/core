use ed25519_dalek::{Signer as DalekSigner, SigningKey};

use primitives::SignerError;

#[derive(Debug)]
pub struct Ed25519KeyPair {
    signing_key: SigningKey,
    pub public_key_bytes: [u8; ed25519_dalek::PUBLIC_KEY_LENGTH],
}

pub fn signing_key_from_bytes(private_key: &[u8]) -> Result<SigningKey, SignerError> {
    let key_bytes: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key.try_into().map_err(|_| SignerError::invalid_input("Invalid Ed25519 private key length"))?;
    Ok(SigningKey::from_bytes(&key_bytes))
}

impl Ed25519KeyPair {
    pub fn from_private_key(private_key: &[u8]) -> Result<Self, SignerError> {
        let signing_key = signing_key_from_bytes(private_key)?;
        Ok(Self {
            public_key_bytes: signing_key.verifying_key().to_bytes(),
            signing_key,
        })
    }

    pub fn sign(&self, digest: &[u8]) -> Vec<u8> {
        self.signing_key.sign(digest).to_bytes().to_vec()
    }
}

pub(crate) fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<ed25519_dalek::Signature, SignerError> {
    let signing_key = signing_key_from_bytes(private_key)?;
    Ok(signing_key.sign(digest))
}

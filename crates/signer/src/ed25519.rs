use ed25519_dalek::{Signer as DalekSigner, SigningKey};

use primitives::SignerError;

#[derive(Debug)]
pub struct Ed25519KeyPair {
    signing_key: SigningKey,
    pub public_key_bytes: [u8; ed25519_dalek::PUBLIC_KEY_LENGTH],
}

impl Ed25519KeyPair {
    pub fn from_private_key(private_key: &[u8]) -> Result<Self, SignerError> {
        let key_bytes: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key.try_into().map_err(|_| SignerError::invalid_input("Invalid Ed25519 private key length"))?;
        let signing_key = SigningKey::from_bytes(&key_bytes);
        Ok(Self {
            public_key_bytes: signing_key.verifying_key().to_bytes(),
            signing_key,
        })
    }

    pub fn sign(&self, digest: &[u8]) -> [u8; ed25519_dalek::SIGNATURE_LENGTH] {
        self.signing_key.sign(digest).to_bytes()
    }
}

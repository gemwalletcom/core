use super::AlienError;
use std::fmt::Debug;

pub type SigningAlgorithm = primitives::SigningAlgorithm;

#[uniffi::remote(Enum)]
pub enum SigningAlgorithm {
    Ed25519,
    Secp256k1,
}

#[uniffi::export(with_foreign)]
pub trait AlienSigner: Send + Sync + Debug {
    fn sign_eip712(&self, typed_data_json: String, private_key: Vec<u8>) -> Result<String, AlienError>;
    fn sign(&self, digest: Vec<u8>, algorithm: SigningAlgorithm, private_key: Vec<u8>) -> Result<Vec<u8>, AlienError>;
}

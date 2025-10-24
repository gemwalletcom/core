use std::fmt::Debug;

use crate::GemstoneError;

#[uniffi::export(with_foreign)]
pub trait AlienSigner: Send + Sync + Debug {
    fn sign_eip712(&self, typed_data_json: String, private_key: Vec<u8>) -> Result<String, GemstoneError>;
}

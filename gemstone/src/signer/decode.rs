use crate::GemstoneError;
use primitives::Chain;

#[uniffi::export]
pub fn decode_private_key(chain: Chain, value: String) -> Result<Vec<u8>, GemstoneError> {
    Ok(signer::decode_private_key(&chain, &value)?.to_vec())
}

#[uniffi::export]
pub fn supports_private_key_import(chain: Chain) -> bool {
    signer::supports_private_key_import(&chain)
}

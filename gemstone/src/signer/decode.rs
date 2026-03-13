use crate::GemstoneError;
use primitives::Chain;
use zeroize::Zeroizing;

#[uniffi::export]
pub fn decode_private_key(chain: Chain, value: String) -> Result<Vec<u8>, GemstoneError> {
    Ok(signer::decode_private_key(&chain, &value)?.to_vec())
}

#[uniffi::export]
pub fn encode_private_key(chain: Chain, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let private_key = Zeroizing::new(private_key);
    signer::encode_private_key(&chain, &private_key).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn supports_private_key_import(chain: Chain) -> bool {
    signer::supports_private_key_import(&chain)
}

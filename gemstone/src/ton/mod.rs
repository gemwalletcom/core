use crate::GemstoneError;

pub mod address;
pub mod jetton;

/// Exports functions
#[uniffi::export]
pub fn ton_encode_get_wallet_address(address: String) -> Result<String, GemstoneError> {
    jetton::encode_get_wallet_address_slice(&address).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn ton_decode_jetton_address(base64_data: String, len: u64) -> Result<String, GemstoneError> {
    jetton::decode_data_to_address(&base64_data, len).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn ton_hex_to_base64_address(hex_str: String) -> Result<String, GemstoneError> {
    address::hex_to_base64_address(hex_str).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn ton_base64_to_hex_address(base64_str: String) -> Result<String, GemstoneError> {
    address::base64_to_hex_address(base64_str).map_err(GemstoneError::from)
}

use crate::{EncodingError, EncodingType};

pub fn encode_base58(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

pub fn decode_base58(value: &str) -> Result<Vec<u8>, EncodingError> {
    bs58::decode(value).into_vec().map_err(|e| EncodingError::Invalid(EncodingType::Base58, e.to_string()))
}

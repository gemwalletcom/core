use crate::{EncodingError, EncodingType};
use base64::{Engine, engine::general_purpose};

pub fn encode_base64(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

pub fn decode_base64(value: &str) -> Result<Vec<u8>, EncodingError> {
    general_purpose::STANDARD
        .decode(value)
        .map_err(|e| EncodingError::Invalid(EncodingType::Base64, e.to_string()))
}

pub fn encode_base64_url(bytes: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn decode_base64_url(value: &str) -> Result<Vec<u8>, EncodingError> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|e| EncodingError::Invalid(EncodingType::Base64, e.to_string()))
}

use crate::EncodingError;
use base64::{Engine, engine::general_purpose};

pub fn encode_base64(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

pub fn decode_base64(value: &str) -> Result<Vec<u8>, EncodingError> {
    Ok(general_purpose::STANDARD.decode(value)?)
}

pub fn decode_base64_no_pad(value: &str) -> Result<Vec<u8>, EncodingError> {
    Ok(general_purpose::STANDARD_NO_PAD.decode(value)?)
}

pub fn encode_base64_url(bytes: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn decode_base64_url(value: &str) -> Result<Vec<u8>, EncodingError> {
    Ok(general_purpose::URL_SAFE_NO_PAD.decode(value)?)
}

pub fn decode_base64_url_padded(value: &str) -> Result<Vec<u8>, EncodingError> {
    Ok(general_purpose::URL_SAFE.decode(value)?)
}

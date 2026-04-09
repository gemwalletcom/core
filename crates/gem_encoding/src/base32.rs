use crate::{EncodingError, EncodingType};
use data_encoding::BASE32_NOPAD;

pub fn decode_base32(value: &[u8]) -> Result<Vec<u8>, EncodingError> {
    BASE32_NOPAD.decode(value).map_err(|e| EncodingError::Invalid(EncodingType::Base32, e.to_string()))
}

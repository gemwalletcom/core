mod error;

#[cfg(feature = "base32")]
mod base32;
#[cfg(feature = "base58")]
mod base58;
#[cfg(feature = "base64")]
mod base64;

pub use error::{EncodingError, EncodingType};

#[cfg(feature = "base32")]
pub use crate::base32::{decode_base32, encode_base32};
#[cfg(feature = "base58")]
pub use crate::base58::{decode_base58, encode_base58};
#[cfg(feature = "base64")]
pub use crate::base64::{decode_base64, decode_base64_no_pad, decode_base64_url, decode_base64_url_padded, encode_base64, encode_base64_url};

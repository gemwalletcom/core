use std::fmt;

#[derive(Debug)]
pub enum EncodingError {
    Invalid(EncodingType, String),
}

#[derive(Debug)]
pub enum EncodingType {
    Base32,
    Base58,
    Base64,
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(encoding, msg) => write!(f, "invalid {encoding}: {msg}"),
        }
    }
}

impl fmt::Display for EncodingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base32 => write!(f, "base32"),
            Self::Base58 => write!(f, "base58"),
            Self::Base64 => write!(f, "base64"),
        }
    }
}

impl std::error::Error for EncodingError {}

impl From<EncodingError> for String {
    fn from(err: EncodingError) -> Self {
        err.to_string()
    }
}

#[cfg(feature = "base64")]
impl From<base64::DecodeError> for EncodingError {
    fn from(err: base64::DecodeError) -> Self {
        Self::Invalid(EncodingType::Base64, err.to_string())
    }
}

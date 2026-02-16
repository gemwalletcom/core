use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone)]
pub struct HexError(String);

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for HexError {}

impl From<hex::FromHexError> for HexError {
    fn from(err: hex::FromHexError) -> Self {
        Self(err.to_string())
    }
}

pub fn encode_with_0x(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

pub fn decode_hex(value: &str) -> Result<Vec<u8>, HexError> {
    let stripped = value.trim().strip_prefix("0x").unwrap_or(value.trim());
    if stripped.is_empty() {
        return Ok(vec![]);
    }
    let normalized: Cow<str> = if stripped.len() % 2 == 1 {
        Cow::Owned(format!("0{stripped}"))
    } else {
        Cow::Borrowed(stripped)
    };
    Ok(hex::decode(&*normalized)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_hex_trims_and_strips_prefix() {
        let bytes = decode_hex(" 0x0a0b ").expect("decode");
        assert_eq!(bytes, vec![0x0a, 0x0b]);
    }

    #[test]
    fn decode_hex_pads_odd_length() {
        let bytes = decode_hex("0xa").expect("decode");
        assert_eq!(bytes, vec![0x0a]);
    }

    #[test]
    fn encode_with_0x_adds_prefix() {
        assert_eq!(encode_with_0x(&[0x0a, 0x0b]), "0x0a0b");
        assert_eq!(encode_with_0x(&[]), "0x");
    }
}

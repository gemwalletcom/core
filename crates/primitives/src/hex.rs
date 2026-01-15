use std::fmt;

#[derive(Debug, Clone)]
pub struct HexError(String);

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for HexError {}

pub fn strip_0x(value: &str) -> &str {
    let trimmed = value.trim();
    trimmed.strip_prefix("0x").unwrap_or(trimmed)
}

pub fn decode_hex(value: &str) -> Result<Vec<u8>, HexError> {
    let stripped = strip_0x(value);
    let normalized = if stripped.len() % 2 == 1 {
        format!("0{stripped}")
    } else {
        stripped.to_string()
    };
    ::hex::decode(normalized).map_err(|err| HexError(format!("Invalid hex: {err}")))
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
}

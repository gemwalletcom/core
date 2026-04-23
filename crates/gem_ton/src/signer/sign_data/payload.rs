use gem_encoding::decode_base64;
use primitives::SignerError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TonSignDataPayload {
    Text { text: String },
    Binary { bytes: String },
    Cell { schema: String, cell: String },
}

impl TonSignDataPayload {
    pub fn data(&self) -> &str {
        match self {
            Self::Text { text } => text,
            Self::Binary { bytes } => bytes,
            Self::Cell { cell, .. } => cell,
        }
    }

    pub fn encode(&self) -> Result<(&str, Vec<u8>), SignerError> {
        match self {
            Self::Text { text } => Ok(("txt", text.as_bytes().to_vec())),
            Self::Binary { bytes } => Ok(("bin", decode_base64(bytes).map_err(|_| SignerError::invalid_input("invalid base64"))?)),
            Self::Cell { .. } => Err(SignerError::InvalidInput("Cell payload not supported for sign-data".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_payload_text() {
        let json = r#"{"type":"text","text":"Hello TON"}"#;
        let parsed: TonSignDataPayload = serde_json::from_str(json).unwrap();

        assert_eq!(parsed, TonSignDataPayload::Text { text: "Hello TON".to_string() });
    }

    #[test]
    fn test_parse_payload_binary() {
        let json = r#"{"type":"binary","bytes":"SGVsbG8="}"#;
        let parsed: TonSignDataPayload = serde_json::from_str(json).unwrap();

        assert_eq!(parsed, TonSignDataPayload::Binary { bytes: "SGVsbG8=".to_string() });
    }

}

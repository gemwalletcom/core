use serde::{Deserialize, Serialize};

use crate::error::SignerError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TonSignDataPayload {
    Text { text: String },
    Binary { bytes: String },
    Cell { cell: String },
}

impl TonSignDataPayload {
    pub fn parse(json: &str) -> Result<Self, SignerError> {
        serde_json::from_str(json).map_err(SignerError::from)
    }

    pub fn data(&self) -> &str {
        match self {
            Self::Text { text } => text,
            Self::Binary { bytes } => bytes,
            Self::Cell { cell } => cell,
        }
    }

    pub fn hash(&self) -> Vec<u8> {
        self.data().as_bytes().to_vec()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TonSignDataResponse {
    signature: String,
    public_key: String,
    timestamp: u64,
    domain: String,
    payload: TonSignDataPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonSignMessageData {
    pub payload: serde_json::Value,
    pub domain: String,
}

impl TonSignMessageData {
    pub fn new(payload: serde_json::Value, domain: String) -> Self {
        Self { payload, domain }
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, SignerError> {
        serde_json::from_slice(data).map_err(SignerError::from)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn get_payload(&self) -> Result<TonSignDataPayload, SignerError> {
        let json = serde_json::to_string(&self.payload)?;
        TonSignDataPayload::parse(&json)
    }
}

impl TonSignDataResponse {
    pub fn new(signature: String, public_key: String, timestamp: u64, domain: String, payload: TonSignDataPayload) -> Self {
        Self {
            signature,
            public_key,
            timestamp,
            domain,
            payload,
        }
    }

    pub fn to_json(&self) -> Result<String, SignerError> {
        serde_json::to_string(self).map_err(SignerError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_payload_text() {
        let json = r#"{"type":"text","text":"Hello TON"}"#;
        let parsed = TonSignDataPayload::parse(json).unwrap();

        assert_eq!(parsed, TonSignDataPayload::Text { text: "Hello TON".to_string() });
        assert_eq!(parsed.data(), "Hello TON");
        assert_eq!(parsed.hash(), b"Hello TON".to_vec());
    }

    #[test]
    fn test_parse_payload_binary() {
        let json = r#"{"type":"binary","bytes":"SGVsbG8="}"#;
        let parsed = TonSignDataPayload::parse(json).unwrap();

        assert_eq!(parsed, TonSignDataPayload::Binary { bytes: "SGVsbG8=".to_string() });
    }

    #[test]
    fn test_parse_payload_cell() {
        let json = r#"{"type":"cell","cell":"te6c"}"#;
        let parsed = TonSignDataPayload::parse(json).unwrap();

        assert_eq!(parsed, TonSignDataPayload::Cell { cell: "te6c".to_string() });
    }

    #[test]
    fn test_response_to_json() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };

        let response = TonSignDataResponse::new(
            "c2lnbmF0dXJl".to_string(),
            "cHVibGljS2V5".to_string(),
            1234567890,
            "example.com".to_string(),
            payload,
        );

        let json = response.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["signature"], "c2lnbmF0dXJl");
        assert_eq!(parsed["publicKey"], "cHVibGljS2V5");
        assert_eq!(parsed["timestamp"], 1234567890);
        assert_eq!(parsed["domain"], "example.com");
        assert_eq!(parsed["payload"]["type"], "text");
        assert_eq!(parsed["payload"]["text"], "Hello TON");
    }

    #[test]
    fn test_ton_sign_message_data() {
        let payload = serde_json::json!({"type": "text", "text": "Hello TON"});
        let data = TonSignMessageData::new(payload.clone(), "example.com".to_string());

        let bytes = data.to_bytes();
        let parsed = TonSignMessageData::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.payload, payload);
        assert_eq!(parsed.domain, "example.com");
    }

    #[test]
    fn test_ton_sign_message_data_get_payload() {
        let payload = serde_json::json!({"type": "text", "text": "Hello TON"});
        let data = TonSignMessageData::new(payload, "example.com".to_string());

        let parsed_payload = data.get_payload().unwrap();
        assert_eq!(parsed_payload, TonSignDataPayload::Text { text: "Hello TON".to_string() });
    }
}

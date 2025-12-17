use primitives::SignerError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TonSignDataPayload {
    Text { text: String },
    Binary { bytes: String },
    Cell { cell: String },
}

impl TonSignDataPayload {
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
    pub payload: TonSignDataPayload,
    pub domain: String,
}

impl TonSignMessageData {
    pub fn new(payload: TonSignDataPayload, domain: String) -> Self {
        Self { payload, domain }
    }

    pub fn from_value(payload: serde_json::Value, domain: String) -> Result<Self, SignerError> {
        let payload: TonSignDataPayload = serde_json::from_value(payload).map_err(SignerError::from)?;
        Ok(Self { payload, domain })
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, SignerError> {
        serde_json::from_slice(data).map_err(SignerError::from)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
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
        let parsed: TonSignDataPayload = serde_json::from_str(json).unwrap();

        assert_eq!(parsed, TonSignDataPayload::Text { text: "Hello TON".to_string() });
        assert_eq!(b"Hello TON".to_vec(), parsed.hash());
    }

    #[test]
    fn test_parse_payload_binary() {
        let json = r#"{"type":"binary","bytes":"SGVsbG8="}"#;
        let parsed: TonSignDataPayload = serde_json::from_str(json).unwrap();

        assert_eq!(parsed, TonSignDataPayload::Binary { bytes: "SGVsbG8=".to_string() });
        assert_eq!("SGVsbG8=".as_bytes().to_vec(), parsed.hash());
    }

    #[test]
    fn test_parse_payload_cell() {
        let json = r#"{"type":"cell","cell":"te6c"}"#;
        let parsed: TonSignDataPayload = serde_json::from_str(json).unwrap();

        assert_eq!(parsed, TonSignDataPayload::Cell { cell: "te6c".to_string() });
        assert_eq!("te6c".as_bytes().to_vec(), parsed.hash());
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
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let data = TonSignMessageData::new(payload.clone(), "example.com".to_string());

        let bytes = data.to_bytes();
        let parsed = TonSignMessageData::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.payload, payload);
        assert_eq!(parsed.domain, "example.com");
    }

    #[test]
    fn test_ton_sign_message_data_get_payload() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let data = TonSignMessageData::new(payload, "example.com".to_string());

        assert_eq!(data.payload, TonSignDataPayload::Text { text: "Hello TON".to_string() });
    }
}

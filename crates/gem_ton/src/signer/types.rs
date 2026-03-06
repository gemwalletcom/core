use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use gem_hash::sha2::sha256;
use primitives::SignerError;
use serde::{Deserialize, Serialize};

use crate::address::Address;

const SIGN_DATA_PREFIX: &[u8] = b"\xff\xffton-connect/sign-data/";

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

    pub fn encode(&self) -> Result<(&str, Vec<u8>), SignerError> {
        match self {
            Self::Text { text } => Ok(("txt", text.as_bytes().to_vec())),
            Self::Binary { bytes } => Ok(("bin", BASE64.decode(bytes).map_err(|e| SignerError::InvalidInput(e.to_string()))?)),
            Self::Cell { .. } => Err(SignerError::InvalidInput("Cell payload not supported for sign-data".to_string())),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TonSignDataResponse {
    signature: String,
    public_key: String,
    address: String,
    timestamp: u64,
    domain: String,
    payload: TonSignDataPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonSignMessageData {
    pub payload: TonSignDataPayload,
    pub domain: String,
    pub address: String,
}

impl TonSignMessageData {
    pub fn new(payload: TonSignDataPayload, domain: String, address: String) -> Self {
        Self { payload, domain, address }
    }

    pub fn from_value(payload: serde_json::Value, domain: String, address: String) -> Result<Self, SignerError> {
        let payload: TonSignDataPayload = serde_json::from_value(payload).map_err(SignerError::from)?;
        Ok(Self { payload, domain, address })
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, SignerError> {
        serde_json::from_slice(data).map_err(SignerError::from)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn hash(&self, timestamp: u64) -> Result<Vec<u8>, SignerError> {
        let address = Address::from_base64_url(&self.address).map_err(|e| SignerError::InvalidInput(e.to_string()))?;
        let domain_bytes = self.domain.as_bytes();
        let (type_prefix, payload_bytes) = self.payload.encode()?;

        let mut msg = Vec::new();
        msg.extend_from_slice(SIGN_DATA_PREFIX);
        msg.extend_from_slice(&address.workchain().to_be_bytes());
        msg.extend_from_slice(address.get_hash_part());
        msg.extend_from_slice(&(domain_bytes.len() as u32).to_be_bytes());
        msg.extend_from_slice(domain_bytes);
        msg.extend_from_slice(&timestamp.to_be_bytes());
        msg.extend_from_slice(type_prefix.as_bytes());
        msg.extend_from_slice(&(payload_bytes.len() as u32).to_be_bytes());
        msg.extend_from_slice(&payload_bytes);

        Ok(sha256(&msg).to_vec())
    }
}

pub struct TonSignResult {
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
    pub timestamp: u64,
}

impl TonSignDataResponse {
    pub fn new(signature: String, public_key: String, address: String, timestamp: u64, domain: String, payload: TonSignDataPayload) -> Self {
        Self {
            signature,
            public_key,
            address,
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

    const TEST_ADDRESS: &str = "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg";

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

    #[test]
    fn test_parse_payload_cell() {
        let json = r#"{"type":"cell","cell":"te6c"}"#;
        let parsed: TonSignDataPayload = serde_json::from_str(json).unwrap();

        assert_eq!(parsed, TonSignDataPayload::Cell { cell: "te6c".to_string() });
    }

    #[test]
    fn test_response_to_json() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };

        let response = TonSignDataResponse::new(
            "c2lnbmF0dXJl".to_string(),
            "abcdef01".to_string(),
            "0:58d5c54fbb8488af7eaad0cdc759ca8f6ff79fc9555106c1339b037ec0a40347".to_string(),
            1234567890,
            "example.com".to_string(),
            payload,
        );

        let json = response.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["signature"], "c2lnbmF0dXJl");
        assert_eq!(parsed["publicKey"], "abcdef01");
        assert_eq!(parsed["address"], "0:58d5c54fbb8488af7eaad0cdc759ca8f6ff79fc9555106c1339b037ec0a40347");
        assert_eq!(parsed["timestamp"], 1234567890);
        assert_eq!(parsed["domain"], "example.com");
        assert_eq!(parsed["payload"]["type"], "text");
        assert_eq!(parsed["payload"]["text"], "Hello TON");
    }

    #[test]
    fn test_ton_sign_message_data() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let data = TonSignMessageData::new(payload.clone(), "example.com".to_string(), TEST_ADDRESS.to_string());

        let bytes = data.to_bytes();
        let parsed = TonSignMessageData::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.payload, payload);
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.address, TEST_ADDRESS);
    }

    #[test]
    fn test_build_sign_data_hash() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };
        let data = TonSignMessageData::new(payload, "example.com".to_string(), TEST_ADDRESS.to_string());

        let hash = data.hash(1234567890).unwrap();

        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_build_sign_data_hash_cell_unsupported() {
        let payload = TonSignDataPayload::Cell { cell: "te6c".to_string() };
        let data = TonSignMessageData::new(payload, "example.com".to_string(), TEST_ADDRESS.to_string());

        assert!(data.hash(1234567890).is_err());
    }
}

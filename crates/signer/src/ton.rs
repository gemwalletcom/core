use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::Signer as DalekSigner;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::ed25519::signing_key_from_bytes;
use crate::error::SignerError;

#[derive(Clone, Debug, PartialEq)]
enum TonSignDataType {
    Text,
    Binary,
    Cell,
}

impl TonSignDataType {
    fn as_str(&self) -> &'static str {
        match self {
            TonSignDataType::Text => "text",
            TonSignDataType::Binary => "binary",
            TonSignDataType::Cell => "cell",
        }
    }

    fn data_field(&self) -> &'static str {
        match self {
            TonSignDataType::Text => "text",
            TonSignDataType::Binary => "bytes",
            TonSignDataType::Cell => "cell",
        }
    }
}

#[derive(Deserialize)]
struct TonSignDataPayloadRaw {
    #[serde(rename = "type")]
    payload_type: String,
    text: Option<String>,
    bytes: Option<String>,
    cell: Option<String>,
}

struct TonSignDataPayload {
    payload_type: TonSignDataType,
    data: String,
}

impl TonSignDataPayload {
    fn parse(json: &str) -> Result<Self, SignerError> {
        let raw: TonSignDataPayloadRaw = serde_json::from_str(json)?;

        let (payload_type, data) = match raw.payload_type.as_str() {
            "text" => (TonSignDataType::Text, raw.text.ok_or("Missing text field")?),
            "binary" => (TonSignDataType::Binary, raw.bytes.ok_or("Missing bytes field")?),
            "cell" => (TonSignDataType::Cell, raw.cell.ok_or("Missing cell field")?),
            _ => return Err(SignerError::new(format!("Unknown payload type: {}", raw.payload_type))),
        };

        Ok(Self { payload_type, data })
    }
}

pub struct TonSignDataInput {
    pub domain: String,
    pub payload: Vec<u8>,
}

#[derive(Serialize)]
struct TonSignDataResponse {
    signature: String,
    #[serde(rename = "publicKey")]
    public_key: String,
    timestamp: u64,
    domain: String,
    payload: serde_json::Value,
}

pub fn sign_ton_personal_message(input: TonSignDataInput, private_key: Vec<u8>) -> Result<String, SignerError> {
    let private_key = Zeroizing::new(private_key);
    let payload_str = std::str::from_utf8(&input.payload).map_err(|e| SignerError::new(e.to_string()))?;
    let parsed = TonSignDataPayload::parse(payload_str)?;

    let signing_key = signing_key_from_bytes(&private_key)?;
    let signature = signing_key.sign(parsed.data.as_bytes());

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

    let payload = serde_json::json!({
        "type": parsed.payload_type.as_str(),
        parsed.payload_type.data_field(): parsed.data,
    });

    let response = TonSignDataResponse {
        signature: STANDARD.encode(signature.to_bytes()),
        public_key: STANDARD.encode(signing_key.verifying_key().to_bytes()),
        timestamp,
        domain: input.domain,
        payload,
    };

    Ok(serde_json::to_string(&response)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Verifier;

    const TEST_PRIVATE_KEY: &str = "1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34";

    #[test]
    fn test_parse_payload_text() {
        let json = r#"{"type":"text","text":"Hello TON"}"#;
        let parsed = TonSignDataPayload::parse(json).unwrap();

        assert_eq!(parsed.payload_type, TonSignDataType::Text);
        assert_eq!(parsed.data, "Hello TON");
    }

    #[test]
    fn test_verify_signature() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let text = "Hello TON";
        let input = TonSignDataInput {
            domain: "example.com".to_string(),
            payload: format!(r#"{{"type":"text","text":"{}"}}"#, text).into_bytes(),
        };

        let result_json = sign_ton_personal_message(input, private_key.clone()).unwrap();
        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();

        let signing_key = signing_key_from_bytes(&private_key).unwrap();
        let public_key = signing_key.verifying_key();

        let signature_base64 = result["signature"].as_str().unwrap();
        let signature_bytes = STANDARD.decode(signature_base64).unwrap();
        let signature = ed25519_dalek::Signature::from_slice(&signature_bytes).unwrap();

        assert!(public_key.verify(text.as_bytes(), &signature).is_ok());
        assert_eq!(result["publicKey"].as_str().unwrap(), STANDARD.encode(public_key.to_bytes()));
        assert!(result["timestamp"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_response_format_text() {
        let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
        let input = TonSignDataInput {
            domain: "react-app.walletconnect.com".to_string(),
            payload: r#"{"type":"text","text":"Hello from WalletConnect TON"}"#.as_bytes().to_vec(),
        };

        let result_json = sign_ton_personal_message(input, private_key).unwrap();
        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();

        assert_eq!(result["domain"], "react-app.walletconnect.com");
        assert!(result["timestamp"].as_u64().unwrap() > 0);
        assert_eq!(result["payload"]["type"], "text");
        assert_eq!(result["payload"]["text"], "Hello from WalletConnect TON");
    }
}

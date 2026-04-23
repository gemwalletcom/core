use primitives::SignerError;
use serde::Serialize;

use super::payload::TonSignDataPayload;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TonSignDataResponse {
    signature: String,
    address: String,
    timestamp: u64,
    domain: String,
    payload: TonSignDataPayload,
}

impl TonSignDataResponse {
    pub fn new(signature: String, address: String, timestamp: u64, domain: String, payload: TonSignDataPayload) -> Self {
        Self {
            signature,
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

    #[test]
    fn test_response_to_json() {
        let payload = TonSignDataPayload::Text { text: "Hello TON".to_string() };

        let response = TonSignDataResponse::new(
            "c2lnbmF0dXJl".to_string(),
            "0:58d5c54fbb8488af7eaad0cdc759ca8f6ff79fc9555106c1339b037ec0a40347".to_string(),
            1234567890,
            "example.com".to_string(),
            payload,
        );

        let actual: serde_json::Value = serde_json::from_str(&response.to_json().unwrap()).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/wc_sign_data_response.json")).unwrap();
        assert_eq!(actual, expected);
    }
}

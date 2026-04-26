use serde::Serialize;

use super::payload::TonSignDataPayload;

#[derive(Serialize)]
pub struct TonSignDataResponse {
    pub signature: String,
    pub address: String,
    pub timestamp: u64,
    pub domain: String,
    pub payload: TonSignDataPayload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_to_json() {
        let response = TonSignDataResponse {
            signature: "c2lnbmF0dXJl".to_string(),
            address: "0:58d5c54fbb8488af7eaad0cdc759ca8f6ff79fc9555106c1339b037ec0a40347".to_string(),
            timestamp: 1234567890,
            domain: "example.com".to_string(),
            payload: TonSignDataPayload::Text { text: "Hello TON".to_string() },
        };

        let actual: serde_json::Value = serde_json::from_value(serde_json::to_value(&response).unwrap()).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/wc_sign_data_response.json")).unwrap();
        assert_eq!(actual, expected);
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectResponse {
    pub status: String,
    pub code: String,
    pub error_data: Option<String>,
    pub data: Option<RiskData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskData {
    #[serde(default)]
    pub has_result: Option<bool>,
    #[serde(default)]
    pub risk_level: Option<i32>,
    pub scanned_ts: Option<u64>,
    pub risk_detail: Option<String>, // json string

    pub risk_category: Option<String>,
    pub risk_code: Option<String>,
    pub trust_score: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response() {
        let json = r#"{
            "status": "ERROR",
            "type": "GENERAL",
            "code": "0030002",
            "errorData": "business is not found:app id can't match business",
            "data": null,
            "subData": null,
            "params": null
        }"#;
        let response = serde_json::from_str::<DetectResponse>(json).unwrap();

        assert_eq!(response.status, "ERROR");
        assert_eq!(response.code, "0030002");
        assert_eq!(response.data, None);
        assert!(!response.error_data.unwrap().is_empty());
    }

    #[test]
    fn test_empty_response() {
        let json = r#"{
            "status": "OK",
            "type": "GENERAL",
            "code": "000000000",
            "errorData": null,
            "data": {
                "risk_level": -1,
                "scanned_ts": null,
                "has_result": false,
                "risk_detail": null,
                "request_id": "8c8414f0bb0645afa4bc3670767cfc7b",
                "polling_interval": 60000
            },
            "subData": null,
            "params": null
        }"#;
        let response = serde_json::from_str::<DetectResponse>(json).unwrap();

        assert_eq!(response.status, "OK");
        assert_eq!(response.code, "000000000");
        assert_eq!(response.error_data, None);

        let data = response.data.unwrap();

        assert_eq!(data.has_result, Some(false));
        assert_eq!(data.risk_level, Some(-1));
        assert_eq!(data.risk_detail, None);
    }

    #[test]
    fn test_address_response() {
        let json = r#"{
            "status": "OK",
            "type": "GENERAL",
            "code": "000000000",
            "errorData": null,
            "data": {
                "risk_level": 1,
                "scanned_ts": 1727869054771,
                "has_result": true,
                "risk_detail": "[]",
                "request_id": "d8c401ff8fe84f1e8def321dd551b670",
                "polling_interval": null
            },
            "subData": null,
            "params": null
        }"#;
        let response = serde_json::from_str::<DetectResponse>(json).unwrap();

        assert_eq!(response.status, "OK");
        assert_eq!(response.code, "000000000");
        assert_eq!(response.error_data, None);

        let data = response.data.unwrap();

        assert_eq!(data.has_result, Some(true));
        assert_eq!(data.risk_level, Some(1));
        assert_eq!(data.scanned_ts, Some(1727869054771));
        assert_eq!(data.risk_detail.unwrap(), "[]");
    }

    #[test]
    fn test_token_response_minimal() {
        let json = r#"{
            "status": "OK",
            "type": "GENERAL",
            "code": "000000000",
            "errorData": null,
            "data": {
                "has_result": true,
                "risk_level": 0,
                "result": {
                    "token-symbol": "USDC",
                    "token-name": "USD Coin",
                    "owner-address": "0x0000000000000000000000000000000000000000",
                    "holders-count": "10",
                    "holders": [
                        { "acountAddress": "0xabc", "tokenBalance": "123" },
                        { "acountAddress": "0xdef", "tokenBalance": "456" }
                    ]
                }
            },
            "subData": null,
            "params": null
        }"#;

        let response = serde_json::from_str::<DetectResponse>(json).unwrap();
        assert_eq!(response.status, "OK");
        let data = response.data.unwrap();
        assert_eq!(data.has_result, Some(true));
        assert_eq!(data.risk_level, Some(0));
    }
}

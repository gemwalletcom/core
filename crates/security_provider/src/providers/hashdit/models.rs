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
    pub has_result: bool,
    pub risk_level: i32,
    pub scanned_ts: Option<u64>,
    pub risk_detail: Option<String>, // json string
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

        assert!(!data.has_result);
        assert_eq!(data.risk_level, -1);
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

        assert!(data.has_result);
        assert_eq!(data.risk_level, 1);
        assert_eq!(data.scanned_ts, Some(1727869054771));
        assert_eq!(data.risk_detail.unwrap(), "[]");
    }
}
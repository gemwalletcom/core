use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanAddressResponse {
    pub status: String,
    pub code: String,
    pub error_data: Option<String>,
    pub data: Option<RiskDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanURLResponse {
    pub status: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskDetail {
    pub white_labels: String,
    pub black_labels: String,
    pub risk_level: i32,
    pub trust_score: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_error_response() {
        let json = r#"{
            "status": "ERROR",
            "type": "GENERAL",
            "code": "0030002",
            "errorData": "business is not found:app id can't match business",
            "data": null,
            "subData": null,
            "params": null
        }"#;
        let response = serde_json::from_str::<ScanAddressResponse>(json).unwrap();

        assert_eq!(response.status, "ERROR");
        assert_eq!(response.code, "0030002");
        assert_eq!(response.data, None);
        assert_eq!(response.error_data.unwrap().is_empty(), false);
    }

    #[test]
    fn test_scan_address_response() {
        let json = r#"{
            "status": "OK",
            "type": "GENERAL",
            "code": "000000000",
            "errorData": null,
            "data": {
                "white_labels": "[]",
                "risk_level": 4,
                "scanned_ts": null,
                "black_labels": "[{\"risk_level\":4,\"data_source\":\"blacklist_malicious_address.py_related\",\"ut\":1727382252}]",
                "has_result": false,
                "risk_code": "[]",
                "risk_detail": null,
                "request_id": "4f4112db538d465db25f62c97a8fdbe8",
                "polling_interval": 60000,
                "trust_score": 50,
                "risk_detail_simple": null
            },
            "subData": null,
            "params": null
        }"#;
        let response = serde_json::from_str::<ScanAddressResponse>(json).unwrap();

        assert_eq!(response.status, "OK");
        assert_eq!(response.code, "000000000");
        assert_eq!(response.error_data, None);

        let data = response.data.unwrap();

        assert_eq!(data.white_labels, "[]");
        assert_eq!(data.risk_level, 4);
        assert_eq!(data.trust_score, 50);
    }
}

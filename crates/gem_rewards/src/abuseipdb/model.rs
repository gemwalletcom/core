use serde::{Deserialize, Serialize};

use crate::model::IpCheckResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbuseIPDBResponse {
    pub data: AbuseIPDBData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbuseIPDBData {
    pub ip_address: String,
    pub is_public: bool,
    pub ip_version: i64,
    pub is_whitelisted: Option<bool>,
    pub abuse_confidence_score: i64,
    pub country_code: String,
    pub usage_type: Option<String>,
    pub isp: Option<String>,
    pub domain: Option<String>,
    pub is_tor: bool,
    pub total_reports: i64,
}

impl AbuseIPDBData {
    pub fn as_ip_check_result(&self) -> IpCheckResult {
        IpCheckResult {
            ip_address: self.ip_address.clone(),
            country_code: self.country_code.clone(),
            confidence_score: self.abuse_confidence_score,
            is_tor: self.is_tor,
            is_vpn: false,
            usage_type: self.usage_type.as_deref().and_then(|s| s.parse().ok()).unwrap_or_default(),
            isp: self.isp.clone().unwrap_or_default(),
        }
    }
}

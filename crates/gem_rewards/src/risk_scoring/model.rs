use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct RiskScoreConfig {
    pub fingerprint_match_score: i64,
    pub ip_reuse_score: i64,
    pub isp_model_match_score: i64,
    pub device_id_reuse_score: i64,
    pub ineligible_ip_type_score: i64,
    pub blocked_ip_types: Vec<String>,
    pub blocked_ip_type_penalty: i64,
    pub max_abuse_score: i64,
    pub penalty_isps: Vec<String>,
    pub isp_penalty_score: i64,
    pub verified_user_reduction: i64,
    pub max_allowed_score: i64,
    pub same_referrer_pattern_threshold: i64,
    pub same_referrer_pattern_penalty: i64,
    pub same_referrer_fingerprint_threshold: i64,
    pub same_referrer_fingerprint_penalty: i64,
    pub lookback_days: i64,
    pub high_risk_platform_stores: Vec<String>,
    pub high_risk_platform_store_penalty: i64,
    pub high_risk_countries: Vec<String>,
    pub high_risk_country_penalty: i64,
}

impl Default for RiskScoreConfig {
    fn default() -> Self {
        Self {
            fingerprint_match_score: 100,
            ip_reuse_score: 50,
            isp_model_match_score: 30,
            device_id_reuse_score: 100,
            ineligible_ip_type_score: 100,
            blocked_ip_types: vec!["Data Center".to_string(), "Web Hosting".to_string(), "Transit".to_string()],
            blocked_ip_type_penalty: 100,
            max_abuse_score: 60,
            penalty_isps: vec![],
            isp_penalty_score: 30,
            verified_user_reduction: 30,
            max_allowed_score: 60,
            same_referrer_pattern_threshold: 3,
            same_referrer_pattern_penalty: 40,
            same_referrer_fingerprint_threshold: 2,
            same_referrer_fingerprint_penalty: 60,
            lookback_days: 30,
            high_risk_platform_stores: vec![],
            high_risk_platform_store_penalty: 20,
            high_risk_countries: vec![],
            high_risk_country_penalty: 15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskSignalInput {
    pub username: String,
    pub device_id: i32,
    pub device_platform: String,
    pub device_platform_store: String,
    pub device_os: String,
    pub device_model: String,
    pub device_locale: String,
    pub ip_address: String,
    pub ip_country_code: String,
    pub ip_usage_type: String,
    pub ip_isp: String,
    pub ip_abuse_score: i64,
    pub referrer_verified: bool,
}

impl RiskSignalInput {
    pub fn generate_fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.device_model);
        hasher.update(&self.device_locale);
        hasher.update(&self.ip_isp);
        hasher.update(&self.ip_country_code);
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone)]
pub struct RiskScore {
    pub score: i64,
    pub is_allowed: bool,
    pub fingerprint: String,
    pub breakdown: RiskScoreBreakdown,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RiskScoreBreakdown {
    #[serde(skip_serializing_if = "is_zero")]
    pub abuse_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub fingerprint_match_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub ip_reuse_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub isp_model_match_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub device_id_reuse_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub ineligible_ip_type_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub verified_user_reduction: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub same_referrer_pattern_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub same_referrer_fingerprint_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub platform_store_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub country_score: i64,
}

fn is_zero(value: &i64) -> bool {
    *value == 0
}

impl RiskScoreBreakdown {
    pub fn to_metadata_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_generation() {
        let input = RiskSignalInput {
            username: "user1".to_string(),
            device_id: 1,
            device_platform: "iOS".to_string(),
            device_platform_store: "appStore".to_string(),
            device_os: "18.0".to_string(),
            device_model: "iPhone15,2".to_string(),
            device_locale: "en-US".to_string(),
            ip_address: "192.168.1.1".to_string(),
            ip_country_code: "US".to_string(),
            ip_usage_type: "Fixed Line ISP".to_string(),
            ip_isp: "Comcast".to_string(),
            ip_abuse_score: 0,
            referrer_verified: false,
        };
        let fingerprint = input.generate_fingerprint();
        assert_eq!(fingerprint.len(), 64);
    }
}

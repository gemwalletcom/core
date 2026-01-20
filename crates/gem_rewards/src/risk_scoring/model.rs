use primitives::{IpUsageType, Platform, PlatformStore};
use sha2::{Digest, Sha256};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RiskScoreConfig {
    pub fingerprint_match_penalty_per_referrer: i64,
    pub fingerprint_match_max_penalty: i64,
    pub ip_reuse_score: i64,
    pub isp_model_match_score: i64,
    pub device_id_reuse_penalty_per_referrer: i64,
    pub device_id_reuse_max_penalty: i64,
    pub ineligible_ip_type_score: i64,
    pub blocked_ip_types: Vec<IpUsageType>,
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
    pub same_referrer_device_model_threshold: i64,
    pub same_referrer_device_model_penalty: i64,
    pub device_model_ring_threshold: i64,
    pub device_model_ring_penalty_per_member: i64,
    pub lookback: Duration,
    pub high_risk_platform_stores: Vec<String>,
    pub high_risk_platform_store_penalty: i64,
    pub high_risk_countries: Vec<String>,
    pub high_risk_country_penalty: i64,
    pub high_risk_locales: Vec<String>,
    pub high_risk_locale_penalty: i64,
    pub high_risk_device_models: Vec<String>,
    pub high_risk_device_model_penalty: i64,
    pub velocity_window: Duration,
    pub velocity_divisor: i64,
    pub velocity_penalty: i64,
    pub referral_per_user_daily: i64,
    pub verified_multiplier: i64,
    pub ip_history_penalty_per_abuser: i64,
    pub ip_history_max_penalty: i64,
    pub cross_referrer_device_penalty: i64,
    pub cross_referrer_fingerprint_threshold: i64,
    pub cross_referrer_fingerprint_penalty: i64,
}

impl Default for RiskScoreConfig {
    fn default() -> Self {
        Self {
            fingerprint_match_penalty_per_referrer: 50,
            fingerprint_match_max_penalty: 200,
            ip_reuse_score: 50,
            isp_model_match_score: 30,
            device_id_reuse_penalty_per_referrer: 50,
            device_id_reuse_max_penalty: 200,
            ineligible_ip_type_score: 100,
            blocked_ip_types: vec![IpUsageType::DataCenter, IpUsageType::Hosting],
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
            same_referrer_device_model_threshold: 3,
            same_referrer_device_model_penalty: 50,
            device_model_ring_threshold: 2,
            device_model_ring_penalty_per_member: 40,
            lookback: Duration::from_secs(30 * 86400),
            high_risk_platform_stores: vec![],
            high_risk_platform_store_penalty: 20,
            high_risk_countries: vec![],
            high_risk_country_penalty: 15,
            high_risk_locales: vec![],
            high_risk_locale_penalty: 10,
            high_risk_device_models: vec!["sdk_gphone".to_string(), "(?i)emulator".to_string(), "(?i)simulator".to_string()],
            high_risk_device_model_penalty: 50,
            velocity_window: Duration::from_secs(300),
            velocity_divisor: 2,
            velocity_penalty: 100,
            referral_per_user_daily: 5,
            verified_multiplier: 2,
            ip_history_penalty_per_abuser: 30,
            ip_history_max_penalty: 150,
            cross_referrer_device_penalty: 500,
            cross_referrer_fingerprint_threshold: 2,
            cross_referrer_fingerprint_penalty: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskSignalInput {
    pub username: String,
    pub device_id: i32,
    pub device_platform: Platform,
    pub device_platform_store: PlatformStore,
    pub device_os: String,
    pub device_model: String,
    pub device_locale: String,
    pub device_currency: String,
    pub ip_address: String,
    pub ip_country_code: String,
    pub ip_usage_type: IpUsageType,
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
    pub same_referrer_device_model_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub device_model_ring_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub platform_store_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub country_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub locale_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub high_risk_device_model_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub velocity_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub ip_history_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub cross_referrer_device_score: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub cross_referrer_fingerprint_score: i64,
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
            device_platform: Platform::IOS,
            device_platform_store: PlatformStore::AppStore,
            device_os: "18.0".to_string(),
            device_model: "iPhone15,2".to_string(),
            device_locale: "en-US".to_string(),
            device_currency: "USD".to_string(),
            ip_address: "192.168.1.1".to_string(),
            ip_country_code: "US".to_string(),
            ip_usage_type: IpUsageType::Isp,
            ip_isp: "Comcast".to_string(),
            ip_abuse_score: 0,
            referrer_verified: false,
        };
        assert_eq!(input.generate_fingerprint().len(), 64);
    }
}

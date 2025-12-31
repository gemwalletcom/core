use crate::model::IpCheckResult;
use storage::models::{NewRiskSignalRow, RiskSignalRow};

use super::model::{RiskScore, RiskScoreConfig, RiskSignalInput};
use super::scoring::calculate_risk_score;

#[derive(Debug, Clone)]
pub struct RiskScoringInput {
    pub username: String,
    pub device_id: i32,
    pub device_platform: String,
    pub device_platform_store: String,
    pub device_os: String,
    pub device_model: String,
    pub device_locale: String,
    pub ip_result: IpCheckResult,
    pub referrer_verified: bool,
}

impl RiskScoringInput {
    pub fn to_signal_input(&self) -> RiskSignalInput {
        RiskSignalInput {
            username: self.username.clone(),
            device_id: self.device_id,
            device_platform: self.device_platform.clone(),
            device_platform_store: self.device_platform_store.clone(),
            device_os: self.device_os.clone(),
            device_model: self.device_model.clone(),
            device_locale: self.device_locale.clone(),
            ip_address: self.ip_result.ip_address.clone(),
            ip_country_code: self.ip_result.country_code.clone(),
            ip_usage_type: self.ip_result.usage_type.clone(),
            ip_isp: self.ip_result.isp.clone(),
            ip_abuse_score: self.ip_result.confidence_score,
            referrer_verified: self.referrer_verified,
        }
    }
}

pub struct RiskResult {
    pub score: RiskScore,
    pub signal: NewRiskSignalRow,
}

pub fn evaluate_risk(input: &RiskScoringInput, existing_signals: &[RiskSignalRow], config: &RiskScoreConfig) -> RiskResult {
    let signal_input = input.to_signal_input();
    let score = calculate_risk_score(&signal_input, existing_signals, config);

    let signal = NewRiskSignalRow {
        fingerprint: score.fingerprint.clone(),
        referrer_username: signal_input.username,
        device_id: signal_input.device_id,
        device_platform: signal_input.device_platform,
        device_platform_store: signal_input.device_platform_store,
        device_os: signal_input.device_os,
        device_model: signal_input.device_model,
        device_locale: signal_input.device_locale,
        ip_address: signal_input.ip_address,
        ip_country_code: signal_input.ip_country_code,
        ip_usage_type: signal_input.ip_usage_type,
        ip_isp: signal_input.ip_isp,
        ip_abuse_score: signal_input.ip_abuse_score as i32,
        risk_score: score.score as i32,
        metadata: Some(score.breakdown.to_metadata_json()),
    };

    RiskResult { score, signal }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::IpCheckResult;

    fn create_test_input() -> RiskScoringInput {
        RiskScoringInput {
            username: "user1".to_string(),
            device_id: 1,
            device_platform: "iOS".to_string(),
            device_platform_store: "appStore".to_string(),
            device_os: "18.0".to_string(),
            device_model: "iPhone15,2".to_string(),
            device_locale: "en-US".to_string(),
            ip_result: IpCheckResult {
                ip_address: "192.168.1.1".to_string(),
                country_code: "US".to_string(),
                confidence_score: 0,
                is_tor: false,
                usage_type: "Fixed Line ISP".to_string(),
                isp: "Comcast".to_string(),
            },
            referrer_verified: false,
        }
    }

    #[test]
    fn evaluate_clean_user() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let result = evaluate_risk(&input, &[], &config);

        assert_eq!(result.score.score, 0);
        assert!(result.score.is_allowed);
        assert_eq!(result.signal.referrer_username, "user1");
        assert_eq!(result.signal.device_model, "iPhone15,2");
    }

    #[test]
    fn evaluate_high_abuse_score() {
        let mut input = create_test_input();
        input.ip_result.confidence_score = 60;
        let config = RiskScoreConfig::default();
        let result = evaluate_risk(&input, &[], &config);

        assert_eq!(result.score.score, 60);
        assert!(!result.score.is_allowed);
    }

    #[test]
    fn signal_populated_correctly() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let result = evaluate_risk(&input, &[], &config);

        assert_eq!(result.signal.ip_address, "192.168.1.1");
        assert_eq!(result.signal.ip_isp, "Comcast");
        assert_eq!(result.signal.device_platform, "iOS");
        assert!(!result.signal.fingerprint.is_empty());
    }
}

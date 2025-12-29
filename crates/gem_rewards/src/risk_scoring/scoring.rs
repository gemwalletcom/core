use storage::models::RiskSignalRow;

use super::model::{RiskScoreBreakdown, RiskScoreConfig, RiskScoreResult, RiskSignalInput};

pub fn calculate_risk_score(input: &RiskSignalInput, existing_signals: &[RiskSignalRow], config: &RiskScoreConfig) -> RiskScoreResult {
    let fingerprint = input.generate_fingerprint();
    let mut breakdown = RiskScoreBreakdown {
        abuse_score: input.ip_abuse_score,
        ..Default::default()
    };

    let mut fingerprint_matched = false;
    let mut ip_matched = false;
    let mut isp_model_matched = false;
    let mut device_id_matched = false;

    for signal in existing_signals {
        if signal.username == input.username {
            continue;
        }

        if !fingerprint_matched && signal.fingerprint == fingerprint {
            breakdown.fingerprint_match_score = config.fingerprint_match_score;
            fingerprint_matched = true;
            isp_model_matched = true;
        }

        if !ip_matched && signal.ip_address == input.ip_address {
            breakdown.ip_reuse_score = config.ip_reuse_score;
            ip_matched = true;
        }

        if !isp_model_matched && signal.ip_isp == input.ip_isp && signal.device_model == input.device_model {
            breakdown.isp_model_match_score = config.isp_model_match_score;
            isp_model_matched = true;
        }

        if !device_id_matched && signal.device_id == input.device_id {
            breakdown.device_id_reuse_score = config.device_id_reuse_score;
            device_id_matched = true;
        }

        if fingerprint_matched && ip_matched && isp_model_matched && device_id_matched {
            break;
        }
    }

    let score = breakdown.abuse_score
        + breakdown.fingerprint_match_score
        + breakdown.ip_reuse_score
        + breakdown.isp_model_match_score
        + breakdown.device_id_reuse_score;

    RiskScoreResult {
        score,
        is_allowed: score < config.max_allowed_score,
        fingerprint,
        breakdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_input() -> RiskSignalInput {
        RiskSignalInput {
            username: "user1".to_string(),
            device_id: 1,
            device_platform: "iOS".to_string(),
            device_os: "18.0".to_string(),
            device_model: "iPhone15,2".to_string(),
            device_locale: "en-US".to_string(),
            ip_address: "192.168.1.1".to_string(),
            ip_country_code: "US".to_string(),
            ip_usage_type: "Fixed Line ISP".to_string(),
            ip_isp: "Comcast".to_string(),
            ip_abuse_score: 0,
        }
    }

    fn create_signal(username: &str, fingerprint: &str, ip: &str, isp: &str, model: &str, device_id: i32) -> RiskSignalRow {
        RiskSignalRow {
            id: 1,
            fingerprint: fingerprint.to_string(),
            username: username.to_string(),
            device_id,
            device_platform: "iOS".to_string(),
            device_os: "18.0".to_string(),
            device_model: model.to_string(),
            device_locale: "en-US".to_string(),
            ip_address: ip.to_string(),
            ip_country_code: "US".to_string(),
            ip_usage_type: "Fixed Line ISP".to_string(),
            ip_isp: isp.to_string(),
            ip_abuse_score: 0,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    #[test]
    fn clean_user() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], &config);

        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }

    #[test]
    fn high_abuse_score() {
        let mut input = create_test_input();
        input.ip_abuse_score = 60;
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], &config);

        assert_eq!(result.score, 60);
        assert!(!result.is_allowed);
    }

    #[test]
    fn fingerprint_match() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        let existing = create_signal("other_user", &fingerprint, "10.0.0.1", "Comcast", "iPhone15,2", 2);
        let result = calculate_risk_score(&input, &[existing], &config);

        assert_eq!(result.score, 100);
        assert!(!result.is_allowed);
    }

    #[test]
    fn ip_reuse() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        let existing = create_signal("other_user", "different", "192.168.1.1", "Verizon", "Pixel 8", 2);
        let result = calculate_risk_score(&input, &[existing], &config);

        assert_eq!(result.score, 50);
        assert!(!result.is_allowed);
    }

    #[test]
    fn isp_model_match() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        let existing = create_signal("other_user", "different", "10.0.0.1", "Comcast", "iPhone15,2", 2);
        let result = calculate_risk_score(&input, &[existing], &config);

        assert_eq!(result.score, 30);
        assert!(result.is_allowed);
    }

    #[test]
    fn device_id_reuse() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        let existing = create_signal("other_user", "different", "10.0.0.1", "Verizon", "Pixel 8", 1);
        let result = calculate_risk_score(&input, &[existing], &config);

        assert_eq!(result.score, 100);
        assert!(!result.is_allowed);
    }

    #[test]
    fn same_user_ignored() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        let existing = create_signal("user1", &fingerprint, "192.168.1.1", "Comcast", "iPhone15,2", 1);
        let result = calculate_risk_score(&input, &[existing], &config);

        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }

    #[test]
    fn no_double_counting_fingerprint_and_isp_model() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        let existing = create_signal("other_user", &fingerprint, "10.0.0.1", "Comcast", "iPhone15,2", 2);
        let result = calculate_risk_score(&input, &[existing], &config);

        assert_eq!(result.breakdown.fingerprint_match_score, 100);
        assert_eq!(result.breakdown.isp_model_match_score, 0);
        assert_eq!(result.score, 100);
    }
}

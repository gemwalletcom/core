use storage::models::RiskSignalRow;

use super::model::{RiskScore, RiskScoreBreakdown, RiskScoreConfig, RiskSignalInput};

pub fn calculate_risk_score(input: &RiskSignalInput, existing_signals: &[RiskSignalRow], config: &RiskScoreConfig) -> RiskScore {
    let fingerprint = input.generate_fingerprint();

    let is_penalty_isp = config.penalty_isps.iter().any(|isp| input.ip_isp.contains(isp));
    let is_blocked_type = config.blocked_ip_types.iter().any(|t| input.ip_usage_type.contains(t));

    let mut breakdown = RiskScoreBreakdown {
        abuse_score: if is_blocked_type {
            input.ip_abuse_score
        } else {
            input.ip_abuse_score.min(config.max_abuse_score)
        },
        ineligible_ip_type_score: if is_blocked_type {
            config.blocked_ip_type_penalty
        } else if is_penalty_isp {
            config.isp_penalty_score
        } else {
            0
        },
        verified_user_reduction: if input.referrer_verified { config.verified_user_reduction } else { 0 },
        ..Default::default()
    };

    let mut fingerprint_matched = false;
    let mut ip_matched = false;
    let mut isp_model_matched = false;
    let mut device_id_matched = false;

    for signal in existing_signals {
        if signal.referrer_username == input.username {
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

    let score = (breakdown.abuse_score
        + breakdown.fingerprint_match_score
        + breakdown.ip_reuse_score
        + breakdown.isp_model_match_score
        + breakdown.device_id_reuse_score
        + breakdown.ineligible_ip_type_score
        - breakdown.verified_user_reduction)
        .max(0);

    RiskScore {
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
            referrer_verified: false,
        }
    }

    fn create_signal(referrer_username: &str, fingerprint: &str, ip: &str, isp: &str, model: &str, device_id: i32) -> RiskSignalRow {
        RiskSignalRow {
            id: 1,
            fingerprint: fingerprint.to_string(),
            referrer_username: referrer_username.to_string(),
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
            risk_score: 0,
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
        input.ip_usage_type = "Data Center".to_string();
        input.ip_abuse_score = 70;
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], &config);

        assert_eq!(result.score, 170);
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
        let mut config = RiskScoreConfig::default();
        config.max_allowed_score = 40;

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

    #[test]
    fn default_ip_type_limits_abuse_score() {
        let mut input = create_test_input();
        input.ip_usage_type = "Fixed Line ISP".to_string();
        input.ip_abuse_score = 80;
        let result = calculate_risk_score(&input, &[], &RiskScoreConfig::default());

        assert_eq!(result.breakdown.abuse_score, 60);
        assert!(!result.is_allowed);
    }

    #[test]
    fn blocked_ip_type_gets_full_abuse_score() {
        let mut input = create_test_input();
        input.ip_usage_type = "Data Center/Web Hosting/Transit".to_string();
        input.ip_abuse_score = 60;
        let result = calculate_risk_score(&input, &[], &RiskScoreConfig::default());

        assert_eq!(result.breakdown.abuse_score, 60);
        assert_eq!(result.breakdown.ineligible_ip_type_score, 100);
    }

    #[test]
    fn penalty_isp_adds_points() {
        let mut input = create_test_input();
        input.ip_isp = "SuspiciousISP Inc".to_string();
        input.ip_abuse_score = 25;
        let mut config = RiskScoreConfig::default();
        config.penalty_isps = vec!["SuspiciousISP".to_string()];
        config.max_allowed_score = 50;
        let result = calculate_risk_score(&input, &[], &config);

        assert_eq!(result.breakdown.abuse_score, 25);
        assert_eq!(result.breakdown.ineligible_ip_type_score, 30);
        assert_eq!(result.score, 55);
        assert!(!result.is_allowed);
    }

    #[test]
    fn verified_user_reduces_score() {
        let mut input = create_test_input();
        input.ip_abuse_score = 60;
        input.referrer_verified = true;
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], &config);

        assert_eq!(result.breakdown.abuse_score, 60);
        assert_eq!(result.breakdown.verified_user_reduction, 30);
        assert_eq!(result.score, 30);
        assert!(result.is_allowed);
    }

    #[test]
    fn verified_user_score_cannot_go_negative() {
        let mut input = create_test_input();
        input.referrer_verified = true;
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], &config);

        assert_eq!(result.breakdown.verified_user_reduction, 30);
        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }
}

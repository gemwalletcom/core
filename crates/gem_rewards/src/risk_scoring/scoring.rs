use storage::models::RiskSignalRow;

use super::model::{RiskScore, RiskScoreBreakdown, RiskScoreConfig, RiskSignalInput};

pub fn calculate_risk_score(input: &RiskSignalInput, existing_signals: &[RiskSignalRow], device_model_ring_count: i64, config: &RiskScoreConfig) -> RiskScore {
    let fingerprint = input.generate_fingerprint();

    let is_penalty_isp = config.penalty_isps.iter().any(|isp| input.ip_isp.contains(isp));
    let is_blocked_type = config.blocked_ip_types.iter().any(|t| input.ip_usage_type.contains(t));
    let is_high_risk_platform_store = config.high_risk_platform_stores.iter().any(|s| s == &input.device_platform_store);
    let is_high_risk_country = config.high_risk_countries.iter().any(|c| c == &input.ip_country_code);
    let is_high_risk_locale = config.high_risk_locales.iter().any(|l| l == &input.device_locale);

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
        verified_user_reduction: if input.referrer_verified { -config.verified_user_reduction } else { 0 },
        platform_store_score: if is_high_risk_platform_store {
            config.high_risk_platform_store_penalty
        } else {
            0
        },
        country_score: if is_high_risk_country { config.high_risk_country_penalty } else { 0 },
        locale_score: if is_high_risk_locale { config.high_risk_locale_penalty } else { 0 },
        ..Default::default()
    };

    let mut fingerprint_matched = false;
    let mut ip_matched = false;
    let mut isp_model_matched = false;
    let mut device_id_matched = false;

    let mut same_referrer_pattern_count = 0;
    let mut same_referrer_fingerprint_count = 0;
    let mut same_referrer_device_model_count = 0;

    for signal in existing_signals {
        if signal.referrer_username == input.username {
            if signal.fingerprint == fingerprint {
                same_referrer_fingerprint_count += 1;
            }

            if signal.ip_isp == input.ip_isp && signal.device_model == input.device_model && signal.device_platform == input.device_platform {
                same_referrer_pattern_count += 1;
            }

            if signal.device_model == input.device_model && signal.device_platform == input.device_platform {
                same_referrer_device_model_count += 1;
            }
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

    if same_referrer_fingerprint_count >= config.same_referrer_fingerprint_threshold {
        breakdown.same_referrer_fingerprint_score = config.same_referrer_fingerprint_penalty;
    }

    if same_referrer_pattern_count >= config.same_referrer_pattern_threshold {
        breakdown.same_referrer_pattern_score = config.same_referrer_pattern_penalty;
    }

    if same_referrer_device_model_count >= config.same_referrer_device_model_threshold {
        breakdown.same_referrer_device_model_score = config.same_referrer_device_model_penalty;
    }

    if device_model_ring_count >= config.device_model_ring_threshold {
        breakdown.device_model_ring_score = (device_model_ring_count - 1) * config.device_model_ring_penalty_per_member;
    }

    let score = (breakdown.abuse_score
        + breakdown.fingerprint_match_score
        + breakdown.ip_reuse_score
        + breakdown.isp_model_match_score
        + breakdown.device_id_reuse_score
        + breakdown.ineligible_ip_type_score
        + breakdown.same_referrer_pattern_score
        + breakdown.same_referrer_fingerprint_score
        + breakdown.same_referrer_device_model_score
        + breakdown.device_model_ring_score
        + breakdown.platform_store_score
        + breakdown.country_score
        + breakdown.locale_score
        + breakdown.verified_user_reduction)
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
            device_platform_store: "appStore".to_string(),
            device_os: "18.0".to_string(),
            device_model: "iPhone15,2".to_string(),
            device_locale: "en-US".to_string(),
            device_currency: "USD".to_string(),
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
            device_platform_store: "appStore".to_string(),
            device_os: "18.0".to_string(),
            device_model: model.to_string(),
            device_locale: "en-US".to_string(),
            device_currency: "USD".to_string(),
            ip_address: ip.to_string(),
            ip_country_code: "US".to_string(),
            ip_usage_type: "Fixed Line ISP".to_string(),
            ip_isp: isp.to_string(),
            ip_abuse_score: 0,
            risk_score: 0,
            metadata: None,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    #[test]
    fn clean_user() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], 0, &config);

        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }

    #[test]
    fn high_abuse_score() {
        let mut input = create_test_input();
        input.ip_usage_type = "Data Center".to_string();
        input.ip_abuse_score = 70;
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], 0, &config);

        assert_eq!(result.score, 170);
        assert!(!result.is_allowed);
    }

    #[test]
    fn fingerprint_match() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        let existing = create_signal("other_user", &fingerprint, "10.0.0.1", "Comcast", "iPhone15,2", 2);
        let result = calculate_risk_score(&input, &[existing], 0, &config);

        assert_eq!(result.score, 100);
        assert!(!result.is_allowed);
    }

    #[test]
    fn ip_reuse() {
        let input = create_test_input();
        let mut config = RiskScoreConfig::default();
        config.max_allowed_score = 40;

        let existing = create_signal("other_user", "different", "192.168.1.1", "Verizon", "Pixel 8", 2);
        let result = calculate_risk_score(&input, &[existing], 0, &config);

        assert_eq!(result.score, 50);
        assert!(!result.is_allowed);
    }

    #[test]
    fn isp_model_match() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        let existing = create_signal("other_user", "different", "10.0.0.1", "Comcast", "iPhone15,2", 2);
        let result = calculate_risk_score(&input, &[existing], 0, &config);

        assert_eq!(result.score, 30);
        assert!(result.is_allowed);
    }

    #[test]
    fn device_id_reuse() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        let existing = create_signal("other_user", "different", "10.0.0.1", "Verizon", "Pixel 8", 1);
        let result = calculate_risk_score(&input, &[existing], 0, &config);

        assert_eq!(result.score, 100);
        assert!(!result.is_allowed);
    }

    #[test]
    fn same_user_ignored() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        let existing = create_signal("user1", &fingerprint, "192.168.1.1", "Comcast", "iPhone15,2", 1);
        let result = calculate_risk_score(&input, &[existing], 0, &config);

        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }

    #[test]
    fn no_double_counting_fingerprint_and_isp_model() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        let existing = create_signal("other_user", &fingerprint, "10.0.0.1", "Comcast", "iPhone15,2", 2);
        let result = calculate_risk_score(&input, &[existing], 0, &config);

        assert_eq!(result.breakdown.fingerprint_match_score, 100);
        assert_eq!(result.breakdown.isp_model_match_score, 0);
        assert_eq!(result.score, 100);
    }

    #[test]
    fn default_ip_type_limits_abuse_score() {
        let mut input = create_test_input();
        input.ip_usage_type = "Fixed Line ISP".to_string();
        input.ip_abuse_score = 80;
        let result = calculate_risk_score(&input, &[], 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.abuse_score, 60);
        assert!(!result.is_allowed);
    }

    #[test]
    fn blocked_ip_type_gets_full_abuse_score() {
        let mut input = create_test_input();
        input.ip_usage_type = "Data Center/Web Hosting/Transit".to_string();
        input.ip_abuse_score = 60;
        let result = calculate_risk_score(&input, &[], 0, &RiskScoreConfig::default());

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
        let result = calculate_risk_score(&input, &[], 0, &config);

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
        let result = calculate_risk_score(&input, &[], 0, &config);

        assert_eq!(result.breakdown.abuse_score, 60);
        assert_eq!(result.breakdown.verified_user_reduction, -30);
        assert_eq!(result.score, 30);
        assert!(result.is_allowed);
    }

    #[test]
    fn verified_user_score_cannot_go_negative() {
        let mut input = create_test_input();
        input.referrer_verified = true;
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], 0, &config);

        assert_eq!(result.breakdown.verified_user_reduction, -30);
        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }

    #[test]
    fn same_referrer_pattern_below_threshold() {
        let signals = [
            create_signal("user1", "fp1", "10.0.0.1", "Comcast", "iPhone15,2", 2),
            create_signal("user1", "fp2", "10.0.0.2", "Comcast", "iPhone15,2", 3),
        ];
        let result = calculate_risk_score(&create_test_input(), &signals, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_pattern_score, 0);
        assert!(result.is_allowed);
    }

    #[test]
    fn same_referrer_pattern_at_threshold() {
        let signals = [
            create_signal("user1", "fp1", "10.0.0.1", "Comcast", "iPhone15,2", 2),
            create_signal("user1", "fp2", "10.0.0.2", "Comcast", "iPhone15,2", 3),
            create_signal("user1", "fp3", "10.0.0.3", "Comcast", "iPhone15,2", 4),
        ];
        let result = calculate_risk_score(&create_test_input(), &signals, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_pattern_score, 40);
        assert_eq!(result.breakdown.same_referrer_device_model_score, 50);
        assert_eq!(result.score, 90);
    }

    #[test]
    fn same_referrer_fingerprint_at_threshold() {
        let input = create_test_input();
        let fingerprint = input.generate_fingerprint();
        let signals = [
            create_signal("user1", &fingerprint, "10.0.0.1", "Comcast", "iPhone15,2", 2),
            create_signal("user1", &fingerprint, "10.0.0.2", "Comcast", "iPhone15,2", 3),
        ];

        let result = calculate_risk_score(&input, &signals, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_fingerprint_score, 60);
        assert!(!result.is_allowed);
    }

    #[test]
    fn same_referrer_both_patterns() {
        let input = create_test_input();
        let fingerprint = input.generate_fingerprint();
        let signals = [
            create_signal("user1", &fingerprint, "10.0.0.1", "Comcast", "iPhone15,2", 2),
            create_signal("user1", &fingerprint, "10.0.0.2", "Comcast", "iPhone15,2", 3),
            create_signal("user1", &fingerprint, "10.0.0.3", "Comcast", "iPhone15,2", 4),
        ];

        let result = calculate_risk_score(&input, &signals, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_pattern_score, 40);
        assert_eq!(result.breakdown.same_referrer_fingerprint_score, 60);
        assert_eq!(result.breakdown.same_referrer_device_model_score, 50);
        assert_eq!(result.score, 150);
        assert!(!result.is_allowed);
    }

    #[test]
    fn same_referrer_different_platform_ignored() {
        let mut signal = create_signal("user1", "fp1", "10.0.0.1", "Comcast", "iPhone15,2", 2);
        signal.device_platform = "android".to_string();
        signal.device_platform_store = "googlePlay".to_string();
        let signals = [
            signal,
            create_signal("user1", "fp2", "10.0.0.2", "Comcast", "iPhone15,2", 3),
            create_signal("user1", "fp3", "10.0.0.3", "Comcast", "iPhone15,2", 4),
        ];

        let result = calculate_risk_score(&create_test_input(), &signals, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_pattern_score, 0);
    }

    #[test]
    fn fraud_multiple_devices_same_fingerprint() {
        let input = RiskSignalInput {
            username: "referrer1".to_string(),
            device_model: "TestDevice X".to_string(),
            device_platform: "android".to_string(),
            device_platform_store: "googlePlay".to_string(),
            ip_isp: "Test Mobile ISP".to_string(),
            ip_country_code: "XX".to_string(),
            device_locale: "en".to_string(),
            ..create_test_input()
        };

        let fingerprint = input.generate_fingerprint();
        let signals: Vec<_> = (0..3)
            .map(|i| {
                let mut s = create_signal("referrer1", &fingerprint, "10.20.30.40", "Test Mobile ISP", "TestDevice X", 100 + i);
                s.device_platform = "android".to_string();
                s.device_platform_store = "googlePlay".to_string();
                s
            })
            .collect();

        let result = calculate_risk_score(&input, &signals, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_pattern_score, 40);
        assert_eq!(result.breakdown.same_referrer_fingerprint_score, 60);
        assert_eq!(result.breakdown.same_referrer_device_model_score, 50);
        assert_eq!(result.score, 150);
        assert!(!result.is_allowed);
    }

    #[test]
    fn same_referrer_device_model_below_threshold() {
        let signals = [
            create_signal("user1", "fp1", "10.0.0.1", "ISP_A", "iPhone15,2", 2),
            create_signal("user1", "fp2", "10.0.0.2", "ISP_B", "iPhone15,2", 3),
        ];
        let result = calculate_risk_score(&create_test_input(), &signals, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_device_model_score, 0);
        assert!(result.is_allowed);
    }

    #[test]
    fn same_referrer_device_model_at_threshold() {
        let signals = [
            create_signal("user1", "fp1", "10.0.0.1", "ISP_A", "iPhone15,2", 2),
            create_signal("user1", "fp2", "10.0.0.2", "ISP_B", "iPhone15,2", 3),
            create_signal("user1", "fp3", "10.0.0.3", "ISP_C", "iPhone15,2", 4),
        ];
        let result = calculate_risk_score(&create_test_input(), &signals, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_device_model_score, 50);
        assert_eq!(result.breakdown.same_referrer_pattern_score, 0);
        assert_eq!(result.score, 50);
    }

    #[test]
    fn same_referrer_device_model_vpn_rotation_detected() {
        let input = RiskSignalInput {
            username: "abuser".to_string(),
            device_model: "INFINIX X6525".to_string(),
            device_platform: "android".to_string(),
            device_platform_store: "googlePlay".to_string(),
            device_locale: "in".to_string(),
            ..create_test_input()
        };

        let signals: Vec<_> = [("Comcast", "US"), ("AT&T", "US"), ("Sky Broadband", "GB"), ("BT", "GB"), ("Charter", "US")]
            .iter()
            .enumerate()
            .map(|(i, (isp, _country))| {
                let mut s = create_signal("abuser", &format!("fp{}", i), &format!("10.0.0.{}", i), isp, "INFINIX X6525", 100 + i as i32);
                s.device_platform = "android".to_string();
                s.device_platform_store = "googlePlay".to_string();
                s
            })
            .collect();

        let result = calculate_risk_score(&input, &signals, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_device_model_score, 50);
        assert_eq!(result.breakdown.same_referrer_pattern_score, 0);
        assert_eq!(result.score, 50);
    }

    #[test]
    fn device_model_ring_detected() {
        let input = create_test_input();
        let result = calculate_risk_score(&input, &[], 2, &RiskScoreConfig::default());

        // count=2: (2-1) * 40 = 40
        assert_eq!(result.breakdown.device_model_ring_score, 40);
        assert_eq!(result.score, 40);
        assert!(result.is_allowed);
    }

    #[test]
    fn device_model_ring_scales_with_count() {
        let input = create_test_input();

        // count=3: (3-1) * 40 = 80
        let result = calculate_risk_score(&input, &[], 3, &RiskScoreConfig::default());
        assert_eq!(result.breakdown.device_model_ring_score, 80);
        assert!(!result.is_allowed);

        // count=5: (5-1) * 40 = 160
        let result = calculate_risk_score(&input, &[], 5, &RiskScoreConfig::default());
        assert_eq!(result.breakdown.device_model_ring_score, 160);
        assert!(!result.is_allowed);
    }

    #[test]
    fn device_model_ring_below_threshold() {
        let input = create_test_input();
        let result = calculate_risk_score(&input, &[], 1, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.device_model_ring_score, 0);
        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }
}

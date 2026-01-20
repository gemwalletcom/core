use std::collections::HashSet;
use std::time::Duration;

use regex::Regex;
use storage::models::RiskSignalRow;

use super::model::{RiskScore, RiskScoreBreakdown, RiskScoreConfig, RiskSignalInput};

pub fn calculate_risk_score(
    input: &RiskSignalInput,
    existing_signals: &[RiskSignalRow],
    device_model_ring_count: i64,
    ip_abuser_count: i64,
    config: &RiskScoreConfig,
) -> RiskScore {
    let fingerprint = input.generate_fingerprint();

    let is_penalty_isp = config.penalty_isps.iter().any(|isp| input.ip_isp.contains(isp));
    let is_blocked_type = config.blocked_ip_types.contains(&input.ip_usage_type);
    let is_high_risk_platform_store = config.high_risk_platform_stores.iter().any(|s| s == input.device_platform_store.as_ref());
    let is_high_risk_country = config.high_risk_countries.iter().any(|c| c == &input.ip_country_code);
    let is_high_risk_locale = config.high_risk_locales.iter().any(|l| l == &input.device_locale);
    let is_high_risk_device_model = config
        .high_risk_device_models
        .iter()
        .any(|pattern| Regex::new(pattern).map(|re| re.is_match(&input.device_model)).unwrap_or(false));

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
        platform_store_score: if is_high_risk_platform_store { config.high_risk_platform_store_penalty } else { 0 },
        country_score: if is_high_risk_country { config.high_risk_country_penalty } else { 0 },
        locale_score: if is_high_risk_locale { config.high_risk_locale_penalty } else { 0 },
        high_risk_device_model_score: if is_high_risk_device_model { config.high_risk_device_model_penalty } else { 0 },
        ..Default::default()
    };

    let mut fingerprint_referrers: HashSet<&str> = HashSet::new();
    let mut device_id_referrers: HashSet<&str> = HashSet::new();
    let mut ip_matched = false;
    let mut isp_model_matched = false;

    let mut same_referrer_pattern_count = 0;
    let mut same_referrer_fingerprint_count = 0;
    let mut same_referrer_device_model_count = 0;

    for signal in existing_signals {
        if signal.referrer_username == input.username {
            if signal.fingerprint == fingerprint {
                same_referrer_fingerprint_count += 1;
            }

            if signal.ip_isp == input.ip_isp && signal.device_model == input.device_model && *signal.device_platform == input.device_platform {
                same_referrer_pattern_count += 1;
            }

            if signal.device_model == input.device_model && *signal.device_platform == input.device_platform {
                same_referrer_device_model_count += 1;
            }
            continue;
        }

        // Scaled penalties (count unique referrers)
        if signal.fingerprint == fingerprint {
            fingerprint_referrers.insert(&signal.referrer_username);
        }

        if signal.device_id == input.device_id {
            device_id_referrers.insert(&signal.referrer_username);
        }

        // Binary penalties (first match triggers full penalty)
        if !ip_matched && signal.ip_address == input.ip_address {
            ip_matched = true;
        }

        if !isp_model_matched && signal.ip_isp == input.ip_isp && signal.device_model == input.device_model {
            isp_model_matched = true;
        }
    }

    // Scaled penalties with caps
    let fingerprint_penalty = fingerprint_referrers.len() as i64 * config.fingerprint_match_penalty_per_referrer;
    breakdown.fingerprint_match_score = fingerprint_penalty.min(config.fingerprint_match_max_penalty);

    let device_id_penalty = device_id_referrers.len() as i64 * config.device_id_reuse_penalty_per_referrer;
    breakdown.device_id_reuse_score = device_id_penalty.min(config.device_id_reuse_max_penalty);

    // Binary penalties
    if ip_matched {
        breakdown.ip_reuse_score = config.ip_reuse_score;
    }
    if isp_model_matched && fingerprint_referrers.is_empty() {
        breakdown.isp_model_match_score = config.isp_model_match_score;
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

    if ip_abuser_count > 0 {
        let ip_history_penalty = ip_abuser_count * config.ip_history_penalty_per_abuser;
        breakdown.ip_history_score = ip_history_penalty.min(config.ip_history_max_penalty);
    }

    let same_referrer_signals: Vec<_> = existing_signals.iter().filter(|s| s.referrer_username == input.username).collect();
    let multiplier = if input.referrer_verified { config.verified_multiplier } else { 1 };
    let daily_limit = config.referral_per_user_daily * multiplier;
    let velocity_threshold = daily_limit / config.velocity_divisor.max(1);
    let (signals_in_window, speed_multiplier) = count_signals_in_recent_window(&same_referrer_signals, config.velocity_window);
    if signals_in_window >= velocity_threshold {
        let over_threshold = signals_in_window - velocity_threshold + 1;
        breakdown.velocity_score = ((over_threshold * config.velocity_penalty) as f64 * speed_multiplier) as i64;
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
        + breakdown.high_risk_device_model_score
        + breakdown.velocity_score
        + breakdown.ip_history_score
        + breakdown.verified_user_reduction)
        .max(0);

    RiskScore {
        score,
        is_allowed: score < config.max_allowed_score,
        fingerprint,
        breakdown,
    }
}

fn count_signals_in_recent_window(signals: &[&RiskSignalRow], window: Duration) -> (i64, f64) {
    let now = chrono::Utc::now().naive_utc();
    let window_secs = window.as_secs() as i64;
    let recent: Vec<_> = signals.iter().filter(|s| now.signed_duration_since(s.created_at).num_seconds() <= window_secs).collect();
    let count = recent.len() as i64;
    if count <= 1 {
        return (count, 1.0);
    }
    let timestamps: Vec<_> = recent.iter().map(|s| s.created_at).collect();
    let (min, max) = (timestamps.iter().min().unwrap(), timestamps.iter().max().unwrap());
    let span_secs = max.signed_duration_since(*min).num_seconds().max(1);
    let speed_multiplier = 1.0 + (window_secs - span_secs) as f64 / window_secs as f64;
    (count, speed_multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{IpUsageType, Platform, PlatformStore};

    fn create_test_input() -> RiskSignalInput {
        RiskSignalInput {
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
        }
    }

    fn create_signal(referrer_username: &str, fingerprint: &str, ip: &str, isp: &str, model: &str, device_id: i32) -> RiskSignalRow {
        RiskSignalRow {
            id: 1,
            fingerprint: fingerprint.to_string(),
            referrer_username: referrer_username.to_string(),
            device_id,
            device_platform: Platform::IOS.into(),
            device_platform_store: PlatformStore::AppStore.into(),
            device_os: "18.0".to_string(),
            device_model: model.to_string(),
            device_locale: "en-US".to_string(),
            device_currency: "USD".to_string(),
            ip_address: ip.to_string(),
            ip_country_code: "US".to_string(),
            ip_usage_type: IpUsageType::Isp.into(),
            ip_isp: isp.to_string(),
            ip_abuse_score: 0,
            risk_score: 0,
            metadata: None,
            created_at: chrono::Utc::now().naive_utc() - chrono::TimeDelta::hours(1),
        }
    }

    #[test]
    fn clean_user() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], 0, 0, &config);

        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }

    #[test]
    fn high_abuse_score() {
        let mut input = create_test_input();
        input.ip_usage_type = IpUsageType::DataCenter;
        input.ip_abuse_score = 70;
        let config = RiskScoreConfig::default();
        let result = calculate_risk_score(&input, &[], 0, 0, &config);

        assert_eq!(result.score, 170);
        assert!(!result.is_allowed);
    }

    #[test]
    fn fingerprint_match() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        // 1 referrer * 50 per referrer = 50
        let existing = create_signal("other_user", &fingerprint, "10.0.0.1", "Comcast", "iPhone15,2", 2);
        let result = calculate_risk_score(&input, &[existing], 0, 0, &config);

        assert_eq!(result.score, 50);
        assert!(result.is_allowed);
    }

    #[test]
    fn fingerprint_match_scales_with_referrers() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        // 2 referrers * 50 = 100
        let signals = vec![
            create_signal("referrer_a", &fingerprint, "10.0.0.1", "Comcast", "iPhone15,2", 2),
            create_signal("referrer_b", &fingerprint, "10.0.0.2", "Comcast", "iPhone15,2", 3),
        ];
        let result = calculate_risk_score(&input, &signals, 0, 0, &config);

        assert_eq!(result.breakdown.fingerprint_match_score, 100);
        assert!(!result.is_allowed);
    }

    #[test]
    fn fingerprint_match_capped() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        // 5 referrers * 50 = 250, but capped at 200
        let signals: Vec<_> = (0..5)
            .map(|i| create_signal(&format!("referrer_{}", i), &fingerprint, &format!("10.0.0.{}", i), "Comcast", "iPhone15,2", 10 + i))
            .collect();
        let result = calculate_risk_score(&input, &signals, 0, 0, &config);

        assert_eq!(result.breakdown.fingerprint_match_score, 200);
    }

    #[test]
    fn ip_reuse() {
        let input = create_test_input();
        let config = RiskScoreConfig {
            max_allowed_score: 40,
            ..Default::default()
        };

        let existing = create_signal("other_user", "different", "192.168.1.1", "Verizon", "Pixel 8", 2);
        let result = calculate_risk_score(&input, &[existing], 0, 0, &config);

        assert_eq!(result.score, 50);
        assert!(!result.is_allowed);
    }

    #[test]
    fn isp_model_match() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        let existing = create_signal("other_user", "different", "10.0.0.1", "Comcast", "iPhone15,2", 2);
        let result = calculate_risk_score(&input, &[existing], 0, 0, &config);

        assert_eq!(result.score, 30);
        assert!(result.is_allowed);
    }

    #[test]
    fn device_id_reuse() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        // 1 referrer * 50 per referrer = 50
        let existing = create_signal("other_user", "different", "10.0.0.1", "Verizon", "Pixel 8", 1);
        let result = calculate_risk_score(&input, &[existing], 0, 0, &config);

        assert_eq!(result.score, 50);
        assert!(result.is_allowed);
    }

    #[test]
    fn device_id_reuse_scales_with_referrers() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        // 2 referrers * 50 = 100
        let signals = vec![
            create_signal("referrer_a", "fp1", "10.0.0.1", "Verizon", "Pixel 8", 1),
            create_signal("referrer_b", "fp2", "10.0.0.2", "AT&T", "Galaxy S23", 1),
        ];
        let result = calculate_risk_score(&input, &signals, 0, 0, &config);

        assert_eq!(result.breakdown.device_id_reuse_score, 100);
        assert!(!result.is_allowed);
    }

    #[test]
    fn device_id_reuse_capped() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        // 5 referrers * 50 = 250, but capped at 200
        let signals: Vec<_> = (0..5)
            .map(|i| create_signal(&format!("referrer_{}", i), &format!("fp{}", i), &format!("10.0.0.{}", i), "ISP", "Model", 1))
            .collect();
        let result = calculate_risk_score(&input, &signals, 0, 0, &config);

        assert_eq!(result.breakdown.device_id_reuse_score, 200);
    }

    #[test]
    fn same_user_ignored() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        let existing = create_signal("user1", &fingerprint, "192.168.1.1", "Comcast", "iPhone15,2", 1);
        let result = calculate_risk_score(&input, &[existing], 0, 0, &config);

        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }

    #[test]
    fn no_double_counting_fingerprint_and_isp_model() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();
        let fingerprint = input.generate_fingerprint();

        // When fingerprint matches, isp_model is not counted (fingerprint is more specific)
        let existing = create_signal("other_user", &fingerprint, "10.0.0.1", "Comcast", "iPhone15,2", 2);
        let result = calculate_risk_score(&input, &[existing], 0, 0, &config);

        assert_eq!(result.breakdown.fingerprint_match_score, 50);
        assert_eq!(result.breakdown.isp_model_match_score, 0);
        assert_eq!(result.score, 50);
    }

    #[test]
    fn default_ip_type_limits_abuse_score() {
        let mut input = create_test_input();
        input.ip_usage_type = IpUsageType::Isp;
        input.ip_abuse_score = 80;
        let result = calculate_risk_score(&input, &[], 0, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.abuse_score, 60);
        assert!(!result.is_allowed);
    }

    #[test]
    fn blocked_ip_type_gets_full_abuse_score() {
        let mut input = create_test_input();
        input.ip_usage_type = IpUsageType::DataCenter;
        input.ip_abuse_score = 60;
        let result = calculate_risk_score(&input, &[], 0, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.abuse_score, 60);
        assert_eq!(result.breakdown.ineligible_ip_type_score, 100);
    }

    #[test]
    fn penalty_isp_adds_points() {
        let mut input = create_test_input();
        input.ip_isp = "SuspiciousISP Inc".to_string();
        input.ip_abuse_score = 25;
        let config = RiskScoreConfig {
            penalty_isps: vec!["SuspiciousISP".to_string()],
            max_allowed_score: 50,
            ..Default::default()
        };
        let result = calculate_risk_score(&input, &[], 0, 0, &config);

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
        let result = calculate_risk_score(&input, &[], 0, 0, &config);

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
        let result = calculate_risk_score(&input, &[], 0, 0, &config);

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
        let result = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default());

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
        let result = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default());

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

        let result = calculate_risk_score(&input, &signals, 0, 0, &RiskScoreConfig::default());

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

        let result = calculate_risk_score(&input, &signals, 0, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_pattern_score, 40);
        assert_eq!(result.breakdown.same_referrer_fingerprint_score, 60);
        assert_eq!(result.breakdown.same_referrer_device_model_score, 50);
        assert_eq!(result.score, 150);
        assert!(!result.is_allowed);
    }

    #[test]
    fn same_referrer_different_platform_ignored() {
        let mut signal = create_signal("user1", "fp1", "10.0.0.1", "Comcast", "iPhone15,2", 2);
        signal.device_platform = Platform::Android.into();
        signal.device_platform_store = PlatformStore::GooglePlay.into();
        let signals = [
            signal,
            create_signal("user1", "fp2", "10.0.0.2", "Comcast", "iPhone15,2", 3),
            create_signal("user1", "fp3", "10.0.0.3", "Comcast", "iPhone15,2", 4),
        ];

        let result = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_pattern_score, 0);
    }

    #[test]
    fn fraud_multiple_devices_same_fingerprint() {
        let input = RiskSignalInput {
            username: "referrer1".to_string(),
            device_model: "TestDevice X".to_string(),
            device_platform: Platform::Android,
            device_platform_store: PlatformStore::GooglePlay,
            ip_isp: "Test Mobile ISP".to_string(),
            ip_country_code: "XX".to_string(),
            device_locale: "en".to_string(),
            ..create_test_input()
        };

        let fingerprint = input.generate_fingerprint();
        let signals: Vec<_> = (0..3)
            .map(|i| {
                let mut s = create_signal("referrer1", &fingerprint, "10.20.30.40", "Test Mobile ISP", "TestDevice X", 100 + i);
                s.device_platform = Platform::Android.into();
                s.device_platform_store = PlatformStore::GooglePlay.into();
                s
            })
            .collect();

        let result = calculate_risk_score(&input, &signals, 0, 0, &RiskScoreConfig::default());

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
        let result = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default());

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
        let result = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_device_model_score, 50);
        assert_eq!(result.breakdown.same_referrer_pattern_score, 0);
        assert_eq!(result.score, 50);
    }

    #[test]
    fn same_referrer_device_model_vpn_rotation_detected() {
        let input = RiskSignalInput {
            username: "abuser".to_string(),
            device_model: "INFINIX X6525".to_string(),
            device_platform: Platform::Android,
            device_platform_store: PlatformStore::GooglePlay,
            device_locale: "in".to_string(),
            ..create_test_input()
        };

        let signals: Vec<_> = [("Comcast", "US"), ("AT&T", "US"), ("Sky Broadband", "GB"), ("BT", "GB"), ("Charter", "US")]
            .iter()
            .enumerate()
            .map(|(i, (isp, _country))| {
                let mut s = create_signal("abuser", &format!("fp{}", i), &format!("10.0.0.{}", i), isp, "INFINIX X6525", 100 + i as i32);
                s.device_platform = Platform::Android.into();
                s.device_platform_store = PlatformStore::GooglePlay.into();
                s
            })
            .collect();

        let result = calculate_risk_score(&input, &signals, 0, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.same_referrer_device_model_score, 50);
        assert_eq!(result.breakdown.same_referrer_pattern_score, 0);
        assert_eq!(result.score, 50);
    }

    #[test]
    fn device_model_ring_detected() {
        let input = create_test_input();
        let result = calculate_risk_score(&input, &[], 2, 0, &RiskScoreConfig::default());

        // count=2: (2-1) * 40 = 40
        assert_eq!(result.breakdown.device_model_ring_score, 40);
        assert_eq!(result.score, 40);
        assert!(result.is_allowed);
    }

    #[test]
    fn device_model_ring_scales_with_count() {
        let input = create_test_input();

        // count=3: (3-1) * 40 = 80
        let result = calculate_risk_score(&input, &[], 3, 0, &RiskScoreConfig::default());
        assert_eq!(result.breakdown.device_model_ring_score, 80);
        assert!(!result.is_allowed);

        // count=5: (5-1) * 40 = 160
        let result = calculate_risk_score(&input, &[], 5, 0, &RiskScoreConfig::default());
        assert_eq!(result.breakdown.device_model_ring_score, 160);
        assert!(!result.is_allowed);
    }

    #[test]
    fn device_model_ring_below_threshold() {
        let input = create_test_input();
        let result = calculate_risk_score(&input, &[], 1, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.device_model_ring_score, 0);
        assert_eq!(result.score, 0);
        assert!(result.is_allowed);
    }

    fn create_recent_signal(referrer_username: &str, seconds_ago: i64) -> RiskSignalRow {
        let mut s = create_signal(referrer_username, "fp", "10.0.0.1", "ISP", "Model", 2);
        s.created_at = chrono::Utc::now().naive_utc() - chrono::TimeDelta::seconds(seconds_ago);
        s
    }

    #[test]
    fn velocity_no_burst() {
        // Signals from different referrer don't trigger velocity for user1
        let result = calculate_risk_score(&create_test_input(), &[create_recent_signal("other", 60)], 0, 0, &RiskScoreConfig::default());
        assert_eq!(result.breakdown.velocity_score, 0);
    }

    #[test]
    fn velocity_burst() {
        // Normal user threshold=2 (5/2), 1 signal - no penalty
        let result = calculate_risk_score(&create_test_input(), &[create_recent_signal("user1", 60)], 0, 0, &RiskScoreConfig::default());
        assert_eq!(result.breakdown.velocity_score, 0);
        // 2 signals triggers penalty
        let signals = vec![create_recent_signal("user1", 60), create_recent_signal("user1", 120)];
        let result = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default());
        assert!(result.breakdown.velocity_score > 0);
    }

    #[test]
    fn velocity_scales_with_count_and_speed() {
        // More signals and tighter span = higher penalty
        let signals = vec![create_recent_signal("user1", 60), create_recent_signal("user1", 120)];
        let score2 = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default())
            .breakdown
            .velocity_score;
        let signals = vec![create_recent_signal("user1", 60), create_recent_signal("user1", 120), create_recent_signal("user1", 180)];
        let score3 = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default())
            .breakdown
            .velocity_score;
        assert!(score3 > score2);
        assert!(score2 > 0);
    }

    #[test]
    fn velocity_faster_spam_higher_penalty() {
        // Same count but tighter time = higher penalty
        // 3 signals in 120s span: multiplier=1.6, penalty=300*1.6=480
        let signals = vec![create_recent_signal("user1", 60), create_recent_signal("user1", 120), create_recent_signal("user1", 180)];
        let slow = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default())
            .breakdown
            .velocity_score;
        // 3 signals in 20s span: multiplier=1+(300-20)/300=1.93, penalty=300*1.93=579
        let signals = vec![create_recent_signal("user1", 60), create_recent_signal("user1", 70), create_recent_signal("user1", 80)];
        let fast = calculate_risk_score(&create_test_input(), &signals, 0, 0, &RiskScoreConfig::default())
            .breakdown
            .velocity_score;
        assert!(fast > slow);
    }

    #[test]
    fn velocity_verified_user() {
        let mut input = create_test_input();
        input.referrer_verified = true;
        // Verified user threshold=5 (10/2), 4 signals - no penalty
        let signals: Vec<_> = (0..4).map(|i| create_recent_signal("user1", 60 + i * 30)).collect();
        assert_eq!(calculate_risk_score(&input, &signals, 0, 0, &RiskScoreConfig::default()).breakdown.velocity_score, 0);
        // 5 signals triggers penalty
        let signals: Vec<_> = (0..5).map(|i| create_recent_signal("user1", 60 + i * 30)).collect();
        assert!(calculate_risk_score(&input, &signals, 0, 0, &RiskScoreConfig::default()).breakdown.velocity_score > 0);
    }

    #[test]
    fn high_risk_device_model_emulator() {
        let mut input = create_test_input();
        input.device_model = "Google sdk_gphone64_arm64".to_string();
        let result = calculate_risk_score(&input, &[], 0, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.high_risk_device_model_score, 50);
        assert_eq!(result.score, 50);
    }

    #[test]
    fn high_risk_device_model_no_match() {
        let input = create_test_input();
        let result = calculate_risk_score(&input, &[], 0, 0, &RiskScoreConfig::default());

        assert_eq!(result.breakdown.high_risk_device_model_score, 0);
    }

    #[test]
    fn high_risk_device_model_custom_pattern() {
        let mut input = create_test_input();
        input.device_model = "INFINIX X6525".to_string();
        let config = RiskScoreConfig {
            high_risk_device_models: vec!["INFINIX".to_string()],
            ..Default::default()
        };
        let result = calculate_risk_score(&input, &[], 0, 0, &config);

        assert_eq!(result.breakdown.high_risk_device_model_score, 50);
    }

    #[test]
    fn high_risk_device_model_regex_pattern() {
        let mut input = create_test_input();
        input.device_model = "Redmi 2201117TY".to_string();
        let config = RiskScoreConfig {
            high_risk_device_models: vec![r"\d{7}[A-Z]{2}".to_string()],
            ..Default::default()
        };
        let result = calculate_risk_score(&input, &[], 0, 0, &config);

        assert_eq!(result.breakdown.high_risk_device_model_score, 50);
    }

    #[test]
    fn ip_history() {
        let input = create_test_input();
        let config = RiskScoreConfig::default();

        assert_eq!(calculate_risk_score(&input, &[], 0, 3, &config).breakdown.ip_history_score, 90);
        assert_eq!(calculate_risk_score(&input, &[], 0, 10, &config).breakdown.ip_history_score, 150);
    }
}

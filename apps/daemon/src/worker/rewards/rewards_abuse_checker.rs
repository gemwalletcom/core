use gem_tracing::info_with_fields;
use primitives::{ConfigKey, NaiveDateTimeExt, now};
use std::error::Error;
use storage::{AbusePatterns, ConfigRepository, Database, RiskSignalsRepository};
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

struct AbuseDetectionConfig {
    disable_threshold: i64,
    attempt_penalty: i64,
    verified_threshold_multiplier: f64,
    lookback: std::time::Duration,
    min_referrals_to_evaluate: i64,
    country_rotation_threshold: i64,
    country_rotation_penalty: i64,
    ring_referrers_per_device_threshold: i64,
    ring_referrers_per_fingerprint_threshold: i64,
    ring_penalty: i64,
    device_farming_threshold: i64,
    device_farming_penalty: i64,
}

struct AbuseEvaluation {
    username: String,
    verified: bool,
    referrals: i64,
    attempts: i64,
    risk_score: i64,
    patterns: AbusePatterns,
    pattern_penalty: f64,
    threshold: f64,
    abuse_score: f64,
    abuse_percent: f64,
}

pub struct RewardsAbuseChecker {
    database: Database,
    stream_producer: StreamProducer,
}

impl RewardsAbuseChecker {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn check(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;
        let config = Self::load_config(client.config())?;
        let since = now().ago(config.lookback);

        let usernames = client.get_referrer_usernames_with_referrals(since, config.min_referrals_to_evaluate)?;

        let mut evaluations: Vec<AbuseEvaluation> = usernames.iter().filter_map(|username| self.evaluate_user(username, &config).ok()).collect();

        evaluations.sort_by(|a, b| b.abuse_percent.partial_cmp(&a.abuse_percent).unwrap_or(std::cmp::Ordering::Equal));

        let (high_risk, low_risk): (Vec<_>, Vec<_>) = evaluations.iter().partition(|e| e.abuse_percent >= 50.0);

        for eval in &high_risk {
            Self::log_evaluation(eval);
        }

        if !low_risk.is_empty() {
            let summary: Vec<String> = low_risk.iter().map(|e| format!("{}({:.0}%)", e.username, e.abuse_percent)).collect();
            info_with_fields!("abuse evaluation summary", users = summary.join(", "));
        }

        let mut disabled_count = 0;
        for eval in evaluations {
            if eval.abuse_score >= eval.threshold
                && let Some(event_id) = self.disable_user(&eval)?
            {
                self.stream_producer
                    .publish_rewards_events(vec![RewardsNotificationPayload::new(event_id)])
                    .await?;
                disabled_count += 1;
            }
        }

        Ok(disabled_count)
    }

    fn evaluate_user(&self, username: &str, config: &AbuseDetectionConfig) -> Result<AbuseEvaluation, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;

        let is_verified = client.rewards().is_verified_by_username(username)?;
        let since = now().ago(config.lookback);

        let referral_count = client.rewards().count_referrals_since(username, since)?;
        let attempt_count = client.count_attempts_for_referrer(username, since)?;
        let risk_score_sum = client.sum_risk_scores_for_referrer(username, since)?;

        let patterns = client.get_abuse_patterns_for_referrer(username, since)?;
        let pattern_penalty = calculate_pattern_penalty(&patterns, config);

        let abuse_score = calculate_abuse_score(risk_score_sum, attempt_count, referral_count, config) + pattern_penalty;
        let threshold = calculate_abuse_threshold(config, is_verified);
        let abuse_percent = (abuse_score / threshold * 100.0).min(100.0);

        Ok(AbuseEvaluation {
            username: username.to_string(),
            verified: is_verified,
            referrals: referral_count,
            attempts: attempt_count,
            risk_score: risk_score_sum,
            patterns,
            pattern_penalty,
            threshold,
            abuse_score,
            abuse_percent,
        })
    }

    fn log_evaluation(eval: &AbuseEvaluation) {
        info_with_fields!(
            "abuse evaluation",
            username = eval.username,
            verified = eval.verified.to_string(),
            referrals = eval.referrals.to_string(),
            attempts = eval.attempts.to_string(),
            risk_score = eval.risk_score.to_string(),
            countries_per_device = eval.patterns.max_countries_per_device.to_string(),
            referrers_per_device = eval.patterns.max_referrers_per_device.to_string(),
            referrers_per_fingerprint = eval.patterns.max_referrers_per_fingerprint.to_string(),
            devices_per_ip = eval.patterns.max_devices_per_ip.to_string(),
            pattern_penalty = format!("{:.0}", eval.pattern_penalty),
            abuse_threshold = format!("{:.0}", eval.threshold),
            abuse_score = format!("{:.0}", eval.abuse_score),
            abuse_percent = format!("{:.0}%", eval.abuse_percent)
        );
    }

    fn disable_user(&self, eval: &AbuseEvaluation) -> Result<Option<i32>, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;

        info_with_fields!(
            "disabled user for abuse",
            username = eval.username,
            abuse_score = format!("{:.0}", eval.abuse_score),
            threshold = format!("{:.0}", eval.threshold)
        );

        let reason = "Auto-disabled due to abuse detection";
        let comment = format!(
            "abuse_score={:.0}, threshold={:.0}, risk_scores={}, attempts={}, referrals={}, countries/device={}, referrers/device={}, referrers/fingerprint={}, devices/ip={}",
            eval.abuse_score,
            eval.threshold,
            eval.risk_score,
            eval.attempts,
            eval.referrals,
            eval.patterns.max_countries_per_device,
            eval.patterns.max_referrers_per_device,
            eval.patterns.max_referrers_per_fingerprint,
            eval.patterns.max_devices_per_ip
        );
        let event_id = client.rewards().disable_rewards(&eval.username, reason, &comment)?;

        Ok(Some(event_id))
    }

    fn load_config(config: &mut dyn ConfigRepository) -> Result<AbuseDetectionConfig, storage::DatabaseError> {
        Ok(AbuseDetectionConfig {
            disable_threshold: config.get_config_i64(ConfigKey::ReferralAbuseDisableThreshold)?,
            attempt_penalty: config.get_config_i64(ConfigKey::ReferralAbuseAttemptPenalty)?,
            verified_threshold_multiplier: config.get_config_f64(ConfigKey::ReferralAbuseVerifiedThresholdMultiplier)?,
            lookback: config.get_config_duration(ConfigKey::ReferralAbuseLookback)?,
            min_referrals_to_evaluate: config.get_config_i64(ConfigKey::ReferralAbuseMinReferralsToEvaluate)?,
            country_rotation_threshold: config.get_config_i64(ConfigKey::ReferralAbuseCountryRotationThreshold)?,
            country_rotation_penalty: config.get_config_i64(ConfigKey::ReferralAbuseCountryRotationPenalty)?,
            ring_referrers_per_device_threshold: config.get_config_i64(ConfigKey::ReferralAbuseRingReferrersPerDeviceThreshold)?,
            ring_referrers_per_fingerprint_threshold: config.get_config_i64(ConfigKey::ReferralAbuseRingReferrersPerFingerprintThreshold)?,
            ring_penalty: config.get_config_i64(ConfigKey::ReferralAbuseRingPenalty)?,
            device_farming_threshold: config.get_config_i64(ConfigKey::ReferralAbuseDeviceFarmingThreshold)?,
            device_farming_penalty: config.get_config_i64(ConfigKey::ReferralAbuseDeviceFarmingPenalty)?,
        })
    }
}

fn calculate_abuse_score(risk_score_sum: i64, attempt_count: i64, referral_count: i64, config: &AbuseDetectionConfig) -> f64 {
    let referrals = referral_count.max(1) as f64;
    let risk_score_per_referral = risk_score_sum as f64 / referrals;
    let attempts_per_referral = attempt_count as f64 / referrals;
    risk_score_per_referral + (attempts_per_referral * config.attempt_penalty as f64)
}

fn calculate_abuse_threshold(config: &AbuseDetectionConfig, is_verified: bool) -> f64 {
    if is_verified {
        config.disable_threshold as f64 * config.verified_threshold_multiplier
    } else {
        config.disable_threshold as f64
    }
}

fn calculate_pattern_penalty(patterns: &AbusePatterns, config: &AbuseDetectionConfig) -> f64 {
    let mut penalty = 0.0;

    if patterns.max_countries_per_device >= config.country_rotation_threshold {
        penalty += config.country_rotation_penalty as f64;
    }

    if patterns.max_referrers_per_device >= config.ring_referrers_per_device_threshold
        || patterns.max_referrers_per_fingerprint >= config.ring_referrers_per_fingerprint_threshold
    {
        penalty += config.ring_penalty as f64;
    }

    if patterns.max_devices_per_ip >= config.device_farming_threshold {
        penalty += config.device_farming_penalty as f64;
    }

    penalty
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AbuseDetectionConfig {
        AbuseDetectionConfig {
            disable_threshold: 200,
            attempt_penalty: 15,
            verified_threshold_multiplier: 2.0,
            lookback: std::time::Duration::from_secs(7 * 86400),
            min_referrals_to_evaluate: 2,
            country_rotation_threshold: 2,
            country_rotation_penalty: 50,
            ring_referrers_per_device_threshold: 2,
            ring_referrers_per_fingerprint_threshold: 2,
            ring_penalty: 80,
            device_farming_threshold: 5,
            device_farming_penalty: 10,
        }
    }

    #[test]
    fn test_abuse_score() {
        assert_eq!(calculate_abuse_score(100, 5, 1, &config()), 175.0);
        assert_eq!(calculate_abuse_score(0, 10, 1, &config()), 150.0);
        assert_eq!(calculate_abuse_score(200, 0, 1, &config()), 200.0);
        assert_eq!(calculate_abuse_score(100, 5, 10, &config()), 17.5);
        assert_eq!(calculate_abuse_score(0, 10, 10, &config()), 15.0);
        assert_eq!(calculate_abuse_score(200, 0, 10, &config()), 20.0);
    }

    #[test]
    fn test_abuse_threshold() {
        assert_eq!(calculate_abuse_threshold(&config(), false), 200.0);
        assert_eq!(calculate_abuse_threshold(&config(), true), 400.0);
    }

    #[test]
    fn test_pattern_penalty() {
        let config = config();
        let base = AbusePatterns {
            max_countries_per_device: 1,
            max_referrers_per_device: 1,
            max_referrers_per_fingerprint: 1,
            max_devices_per_ip: 2,
        };

        assert_eq!(calculate_pattern_penalty(&base, &config), 0.0);
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    max_countries_per_device: 2,
                    ..base
                },
                &config
            ),
            50.0
        );
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    max_referrers_per_device: 2,
                    ..base
                },
                &config
            ),
            80.0
        );
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    max_referrers_per_fingerprint: 2,
                    ..base
                },
                &config
            ),
            80.0
        );
        assert_eq!(calculate_pattern_penalty(&AbusePatterns { max_devices_per_ip: 5, ..base }, &config), 10.0);
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    max_countries_per_device: 5,
                    max_referrers_per_device: 4,
                    max_referrers_per_fingerprint: 3,
                    max_devices_per_ip: 10
                },
                &config
            ),
            140.0
        );
    }
}

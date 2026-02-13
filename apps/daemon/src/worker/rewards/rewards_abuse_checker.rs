use gem_tracing::info_with_fields;
use primitives::rewards::RewardStatus;
use primitives::{ConfigKey, NaiveDateTimeExt, now};
use std::error::Error;
use storage::{AbusePatterns, ConfigCacher, Database, RiskSignalsRepository};
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
    velocity_window: std::time::Duration,
    velocity_divisor: i64,
    velocity_penalty: i64,
    referral_per_user_daily: i64,
    verified_multiplier: i64,
    trusted_multiplier: i64,
    disabled_referrer_penalty: i64,
}

struct AbuseEvaluation {
    username: String,
    status: RewardStatus,
    referrals: i64,
    attempts: i64,
    risk_score: i64,
    patterns: AbusePatterns,
    score: AbuseScoreBreakdown,
    pattern_penalty: PatternPenaltyBreakdown,
    referrer_disabled: bool,
    disabled_referrer_penalty: f64,
    threshold: f64,
    abuse_score: f64,
    abuse_percent: f64,
}

struct AbuseScoreBreakdown {
    base_score: f64,
    risk_score_per_referral: f64,
    attempts_per_referral: f64,
    attempt_penalty_score: f64,
}

struct PatternPenaltyBreakdown {
    country_rotation_penalty: f64,
    ring_penalty: f64,
    device_farming_penalty: f64,
    velocity_penalty: f64,
}

impl PatternPenaltyBreakdown {
    fn total(&self) -> f64 {
        self.country_rotation_penalty + self.ring_penalty + self.device_farming_penalty + self.velocity_penalty
    }
}

pub struct RewardsAbuseChecker {
    database: Database,
    config: ConfigCacher,
    stream_producer: StreamProducer,
}

impl RewardsAbuseChecker {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        let config = ConfigCacher::new(database.clone());
        Self {
            database,
            config,
            stream_producer,
        }
    }

    pub async fn check(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;
        let config = self.load_config()?;
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
                self.stream_producer.publish_rewards_events(vec![RewardsNotificationPayload::new(event_id)]).await?;
                disabled_count += 1;
            }
        }

        Ok(disabled_count)
    }

    fn evaluate_user(&self, username: &str, config: &AbuseDetectionConfig) -> Result<AbuseEvaluation, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;

        let status = client.rewards().get_status_by_username(username)?;
        let since = now().ago(config.lookback);
        let velocity_window_secs = config.velocity_window.as_secs() as i64;

        let referral_count = client.rewards().count_referrals_since(username, since)?;
        let attempt_count = client.count_attempts_for_referrer(username, since)?;
        let risk_score_sum = client.sum_risk_scores_for_referrer(username, since)?;

        let patterns = client.get_abuse_patterns_for_referrer(username, since, velocity_window_secs)?;
        let score = calculate_abuse_score_breakdown(risk_score_sum, attempt_count, referral_count, config);
        let pattern_penalty = calculate_pattern_penalty_breakdown(&patterns, config, &status);

        let referrer_disabled = client
            .rewards()
            .get_referrer_username(username)
            .ok()
            .flatten()
            .and_then(|referrer| client.rewards().get_status_by_username(&referrer).ok())
            .is_some_and(|s| s == RewardStatus::Disabled);
        let disabled_referrer_penalty = if referrer_disabled { config.disabled_referrer_penalty as f64 } else { 0.0 };

        let abuse_score = score.base_score + pattern_penalty.total() + disabled_referrer_penalty;
        let threshold = calculate_abuse_threshold(config, &status);
        let abuse_percent = (abuse_score / threshold * 100.0).min(100.0);

        Ok(AbuseEvaluation {
            username: username.to_string(),
            status,
            referrals: referral_count,
            attempts: attempt_count,
            risk_score: risk_score_sum,
            patterns,
            score,
            pattern_penalty,
            referrer_disabled,
            disabled_referrer_penalty,
            threshold,
            abuse_score,
            abuse_percent,
        })
    }

    fn log_evaluation(eval: &AbuseEvaluation) {
        info_with_fields!(
            "abuse evaluation",
            username = eval.username,
            status = eval.status.as_ref(),
            referrals = eval.referrals.to_string(),
            attempts = eval.attempts.to_string(),
            risk_score = eval.risk_score.to_string(),
            countries_per_device = eval.patterns.max_countries_per_device.to_string(),
            referrers_per_device = eval.patterns.max_referrers_per_device.to_string(),
            referrers_per_fingerprint = eval.patterns.max_referrers_per_fingerprint.to_string(),
            devices_per_ip = eval.patterns.max_devices_per_ip.to_string(),
            velocity_burst = eval.patterns.signals_in_velocity_window.to_string(),
            referrer_disabled = eval.referrer_disabled.to_string(),
            base_score = format!("{:.2}", eval.score.base_score),
            risk_score_per_referral = format!("{:.2}", eval.score.risk_score_per_referral),
            attempts_per_referral = format!("{:.2}", eval.score.attempts_per_referral),
            attempt_penalty_score = format!("{:.2}", eval.score.attempt_penalty_score),
            country_rotation_penalty = format!("{:.0}", eval.pattern_penalty.country_rotation_penalty),
            ring_penalty = format!("{:.0}", eval.pattern_penalty.ring_penalty),
            device_farming_penalty = format!("{:.0}", eval.pattern_penalty.device_farming_penalty),
            velocity_penalty = format!("{:.0}", eval.pattern_penalty.velocity_penalty),
            pattern_penalty = format!("{:.0}", eval.pattern_penalty.total()),
            disabled_referrer_penalty = format!("{:.0}", eval.disabled_referrer_penalty),
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
            "abuse_score={:.0}, threshold={:.0}, base_score={:.2}, risk_scores={}, attempts={}, referrals={}, risk_score/referral={:.2}, attempts/referral={:.2}, attempt_penalty_score={:.2}, pattern_penalty={:.0}, country_rotation_penalty={:.0}, ring_penalty={:.0}, device_farming_penalty={:.0}, velocity_penalty={:.0}, disabled_referrer_penalty={:.0}, countries/device={}, referrers/device={}, referrers/fingerprint={}, devices/ip={}, velocity_burst={}, referrer_disabled={}",
            eval.abuse_score,
            eval.threshold,
            eval.score.base_score,
            eval.risk_score,
            eval.attempts,
            eval.referrals,
            eval.score.risk_score_per_referral,
            eval.score.attempts_per_referral,
            eval.score.attempt_penalty_score,
            eval.pattern_penalty.total(),
            eval.pattern_penalty.country_rotation_penalty,
            eval.pattern_penalty.ring_penalty,
            eval.pattern_penalty.device_farming_penalty,
            eval.pattern_penalty.velocity_penalty,
            eval.disabled_referrer_penalty,
            eval.patterns.max_countries_per_device,
            eval.patterns.max_referrers_per_device,
            eval.patterns.max_referrers_per_fingerprint,
            eval.patterns.max_devices_per_ip,
            eval.patterns.signals_in_velocity_window,
            eval.referrer_disabled
        );
        let event_id = client.rewards().disable_rewards(&eval.username, reason, &comment)?;

        Ok(Some(event_id))
    }

    fn load_config(&self) -> Result<AbuseDetectionConfig, storage::DatabaseError> {
        Ok(AbuseDetectionConfig {
            disable_threshold: self.config.get_i64(ConfigKey::ReferralAbuseDisableThreshold)?,
            attempt_penalty: self.config.get_i64(ConfigKey::ReferralAbuseAttemptPenalty)?,
            verified_threshold_multiplier: self.config.get_f64(ConfigKey::ReferralAbuseVerifiedThresholdMultiplier)?,
            lookback: self.config.get_duration(ConfigKey::ReferralAbuseLookback)?,
            min_referrals_to_evaluate: self.config.get_i64(ConfigKey::ReferralAbuseMinReferralsToEvaluate)?,
            country_rotation_threshold: self.config.get_i64(ConfigKey::ReferralAbuseCountryRotationThreshold)?,
            country_rotation_penalty: self.config.get_i64(ConfigKey::ReferralAbuseCountryRotationPenalty)?,
            ring_referrers_per_device_threshold: self.config.get_i64(ConfigKey::ReferralAbuseRingReferrersPerDeviceThreshold)?,
            ring_referrers_per_fingerprint_threshold: self.config.get_i64(ConfigKey::ReferralAbuseRingReferrersPerFingerprintThreshold)?,
            ring_penalty: self.config.get_i64(ConfigKey::ReferralAbuseRingPenalty)?,
            device_farming_threshold: self.config.get_i64(ConfigKey::ReferralAbuseDeviceFarmingThreshold)?,
            device_farming_penalty: self.config.get_i64(ConfigKey::ReferralAbuseDeviceFarmingPenalty)?,
            velocity_window: self.config.get_duration(ConfigKey::ReferralAbuseVelocityWindow)?,
            velocity_divisor: self.config.get_i64(ConfigKey::ReferralAbuseVelocityDivisor)?,
            velocity_penalty: self.config.get_i64(ConfigKey::ReferralAbuseVelocityPenaltyPerSignal)?,
            referral_per_user_daily: self.config.get_i64(ConfigKey::ReferralPerUserDaily)?,
            verified_multiplier: self.config.get_i64(ConfigKey::ReferralVerifiedMultiplier)?,
            trusted_multiplier: self.config.get_i64(ConfigKey::ReferralTrustedMultiplier)?,
            disabled_referrer_penalty: self.config.get_i64(ConfigKey::ReferralAbuseDisabledReferrerPenalty)?,
        })
    }
}

#[cfg(test)]
fn calculate_abuse_score(risk_score_sum: i64, attempt_count: i64, referral_count: i64, config: &AbuseDetectionConfig) -> f64 {
    calculate_abuse_score_breakdown(risk_score_sum, attempt_count, referral_count, config).base_score
}

fn calculate_abuse_score_breakdown(risk_score_sum: i64, attempt_count: i64, referral_count: i64, config: &AbuseDetectionConfig) -> AbuseScoreBreakdown {
    let referrals = referral_count.max(1) as f64;
    let risk_score_per_referral = risk_score_sum as f64 / referrals;
    let attempts_per_referral = attempt_count as f64 / referrals;
    let attempt_penalty_score = attempts_per_referral * config.attempt_penalty as f64;
    AbuseScoreBreakdown {
        base_score: risk_score_per_referral + attempt_penalty_score,
        risk_score_per_referral,
        attempts_per_referral,
        attempt_penalty_score,
    }
}

fn calculate_abuse_threshold(config: &AbuseDetectionConfig, status: &RewardStatus) -> f64 {
    let multiplier = if *status == RewardStatus::Trusted {
        config.trusted_multiplier as f64
    } else if status.is_verified() {
        config.verified_threshold_multiplier
    } else {
        1.0
    };
    config.disable_threshold as f64 * multiplier
}

#[cfg(test)]
fn calculate_pattern_penalty(patterns: &AbusePatterns, config: &AbuseDetectionConfig, status: &RewardStatus) -> f64 {
    calculate_pattern_penalty_breakdown(patterns, config, status).total()
}

fn calculate_pattern_penalty_breakdown(patterns: &AbusePatterns, config: &AbuseDetectionConfig, status: &RewardStatus) -> PatternPenaltyBreakdown {
    let country_rotation_penalty = if patterns.max_countries_per_device >= config.country_rotation_threshold {
        config.country_rotation_penalty as f64
    } else {
        0.0
    };

    let ring_penalty = if patterns.max_referrers_per_device >= config.ring_referrers_per_device_threshold
        || patterns.max_referrers_per_fingerprint >= config.ring_referrers_per_fingerprint_threshold
    {
        config.ring_penalty as f64
    } else {
        0.0
    };

    let device_farming_penalty = if patterns.max_devices_per_ip >= config.device_farming_threshold {
        config.device_farming_penalty as f64
    } else {
        0.0
    };

    let mut velocity_penalty = 0.0;
    let multiplier = if *status == RewardStatus::Trusted {
        config.trusted_multiplier
    } else if status.is_verified() {
        config.verified_multiplier
    } else {
        1
    };
    let daily_limit = config.referral_per_user_daily * multiplier;
    let velocity_threshold = daily_limit / config.velocity_divisor.max(1);
    if patterns.signals_in_velocity_window >= velocity_threshold {
        let over_threshold = patterns.signals_in_velocity_window - velocity_threshold + 1;
        velocity_penalty = (over_threshold * config.velocity_penalty) as f64;
    }

    PatternPenaltyBreakdown {
        country_rotation_penalty,
        ring_penalty,
        device_farming_penalty,
        velocity_penalty,
    }
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
            velocity_window: std::time::Duration::from_secs(300),
            velocity_divisor: 2,
            velocity_penalty: 100,
            referral_per_user_daily: 5,
            verified_multiplier: 2,
            trusted_multiplier: 3,
            disabled_referrer_penalty: 80,
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
    fn test_abuse_score_breakdown() {
        let score = calculate_abuse_score_breakdown(44, 0, 2, &config());
        assert_eq!(score.risk_score_per_referral, 22.0);
        assert_eq!(score.attempts_per_referral, 0.0);
        assert_eq!(score.attempt_penalty_score, 0.0);
        assert_eq!(score.base_score, 22.0);
    }

    #[test]
    fn test_abuse_threshold() {
        assert_eq!(calculate_abuse_threshold(&config(), &RewardStatus::Unverified), 200.0);
        assert_eq!(calculate_abuse_threshold(&config(), &RewardStatus::Verified), 400.0);
        assert_eq!(calculate_abuse_threshold(&config(), &RewardStatus::Trusted), 600.0);
    }

    #[test]
    fn test_pattern_penalty() {
        let config = config();
        let base = AbusePatterns {
            max_countries_per_device: 1,
            max_referrers_per_device: 1,
            max_referrers_per_fingerprint: 1,
            max_devices_per_ip: 2,
            signals_in_velocity_window: 0,
        };

        assert_eq!(calculate_pattern_penalty(&base, &config, &RewardStatus::Unverified), 0.0);
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    max_countries_per_device: 2,
                    ..base
                },
                &config,
                &RewardStatus::Unverified
            ),
            50.0
        );
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    max_referrers_per_device: 2,
                    ..base
                },
                &config,
                &RewardStatus::Unverified
            ),
            80.0
        );
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    max_referrers_per_fingerprint: 2,
                    ..base
                },
                &config,
                &RewardStatus::Unverified
            ),
            80.0
        );
        assert_eq!(
            calculate_pattern_penalty(&AbusePatterns { max_devices_per_ip: 5, ..base }, &config, &RewardStatus::Unverified),
            10.0
        );

        // Normal user: daily_limit=5, divisor=2, velocity_threshold=2
        // 1 signal doesn't trigger, 2 signals trigger: (2-2+1)*100 = 100
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    signals_in_velocity_window: 1,
                    ..base
                },
                &config,
                &RewardStatus::Unverified
            ),
            0.0
        );
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    signals_in_velocity_window: 2,
                    ..base
                },
                &config,
                &RewardStatus::Unverified
            ),
            100.0
        );

        // Verified user: daily_limit=10, divisor=2, velocity_threshold=5
        // 4 signals don't trigger, 5 signals trigger: (5-5+1)*100 = 100
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    signals_in_velocity_window: 4,
                    ..base
                },
                &config,
                &RewardStatus::Verified
            ),
            0.0
        );
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    signals_in_velocity_window: 5,
                    ..base
                },
                &config,
                &RewardStatus::Verified
            ),
            100.0
        );

        // Trusted user: daily_limit=15, divisor=2, velocity_threshold=7
        // 6 signals don't trigger, 7 signals trigger: (7-7+1)*100 = 100
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    signals_in_velocity_window: 6,
                    ..base
                },
                &config,
                &RewardStatus::Trusted
            ),
            0.0
        );
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    signals_in_velocity_window: 7,
                    ..base
                },
                &config,
                &RewardStatus::Trusted
            ),
            100.0
        );

        // Combined: 50 + 80 + 10 + (5-2+1)*100 = 540
        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    max_countries_per_device: 5,
                    max_referrers_per_device: 4,
                    max_referrers_per_fingerprint: 3,
                    max_devices_per_ip: 10,
                    signals_in_velocity_window: 5,
                },
                &config,
                &RewardStatus::Unverified
            ),
            540.0
        );
    }

    #[test]
    fn test_pattern_penalty_breakdown() {
        let penalties = calculate_pattern_penalty_breakdown(
            &AbusePatterns {
                max_countries_per_device: 2,
                max_referrers_per_device: 2,
                max_referrers_per_fingerprint: 1,
                max_devices_per_ip: 5,
                signals_in_velocity_window: 2,
            },
            &config(),
            &RewardStatus::Unverified,
        );
        assert_eq!(penalties.country_rotation_penalty, 50.0);
        assert_eq!(penalties.ring_penalty, 80.0);
        assert_eq!(penalties.device_farming_penalty, 10.0);
        assert_eq!(penalties.velocity_penalty, 100.0);
        assert_eq!(penalties.total(), 240.0);
    }
}

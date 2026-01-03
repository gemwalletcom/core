use gem_tracing::info_with_fields;
use primitives::{ConfigKey, NaiveDateTimeExt, now};
use std::error::Error;
use storage::{ConfigRepository, Database, RiskSignalsRepository};
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

struct AbuseDetectionConfig {
    disable_threshold: i64,
    attempt_penalty: i64,
    verified_threshold_multiplier: f64,
    lookback_days: i64,
    min_referrals_to_evaluate: i64,
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
        let since = now().days_ago(config.lookback_days);

        let usernames = client.get_referrer_usernames_with_referrals(since, config.min_referrals_to_evaluate)?;

        let mut disabled_count = 0;
        for username in usernames {
            if let Some(event_id) = self.evaluate_and_disable(&username, &config)? {
                self.stream_producer
                    .publish_rewards_events(vec![RewardsNotificationPayload::new(event_id)])
                    .await?;
                disabled_count += 1;
            }
        }

        Ok(disabled_count)
    }

    fn evaluate_and_disable(&self, username: &str, config: &AbuseDetectionConfig) -> Result<Option<i32>, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;

        let is_verified = client.rewards().is_verified_by_username(username)?;
        let since = now().days_ago(config.lookback_days);

        let referral_count = client.rewards().count_referrals_since(username, since)?;
        let attempt_count = client.count_attempts_for_referrer(username, since)?;
        let risk_score_sum = client.sum_risk_scores_for_referrer(username, since)?;

        let abuse_score = calculate_abuse_score(risk_score_sum, attempt_count, referral_count, config);
        let threshold = calculate_abuse_threshold(config, is_verified);

        info_with_fields!(
            "abuse evaluation",
            username = username,
            verified = is_verified.to_string(),
            referrals = referral_count.to_string(),
            attempts = attempt_count.to_string(),
            risk_scores = risk_score_sum.to_string(),
            abuse_score = format!("{:.0}", abuse_score),
            threshold = format!("{:.0}", threshold),
            will_disable = (abuse_score >= threshold).to_string()
        );

        if abuse_score >= threshold {
            info_with_fields!(
                "disabled user for abuse",
                username = username,
                abuse_score = format!("{:.0}", abuse_score),
                threshold = format!("{:.0}", threshold)
            );

            let reason = "Auto-disabled due to abuse detection";
            let comment = format!(
                "abuse_score={:.0}, threshold={:.0}, risk_scores={}, attempts={}, referrals={}",
                abuse_score, threshold, risk_score_sum, attempt_count, referral_count
            );
            let event_id = client.rewards().disable_rewards(username, reason, &comment)?;

            return Ok(Some(event_id));
        }

        Ok(None)
    }

    fn load_config(config: &mut dyn ConfigRepository) -> Result<AbuseDetectionConfig, storage::DatabaseError> {
        Ok(AbuseDetectionConfig {
            disable_threshold: config.get_config_i64(ConfigKey::RewardsAbuseDisableThreshold)?,
            attempt_penalty: config.get_config_i64(ConfigKey::RewardsAbuseAttemptPenalty)?,
            verified_threshold_multiplier: config.get_config_f64(ConfigKey::RewardsAbuseVerifiedThresholdMultiplier)?,
            lookback_days: config.get_config_i64(ConfigKey::RewardsAbuseLookbackDays)?,
            min_referrals_to_evaluate: config.get_config_i64(ConfigKey::RewardsAbuseMinReferralsToEvaluate)?,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AbuseDetectionConfig {
        AbuseDetectionConfig {
            disable_threshold: 200,
            attempt_penalty: 15,
            verified_threshold_multiplier: 2.0,
            lookback_days: 7,
            min_referrals_to_evaluate: 2,
        }
    }

    #[test]
    fn test_abuse_score() {
        // With 1 referral, score = risk_score + attempts * penalty
        assert_eq!(calculate_abuse_score(100, 5, 1, &config()), 175.0);
        assert_eq!(calculate_abuse_score(0, 10, 1, &config()), 150.0);
        assert_eq!(calculate_abuse_score(200, 0, 1, &config()), 200.0);

        // With 10 referrals, score is normalized (divided by 10)
        assert_eq!(calculate_abuse_score(100, 5, 10, &config()), 17.5);
        assert_eq!(calculate_abuse_score(0, 10, 10, &config()), 15.0);
        assert_eq!(calculate_abuse_score(200, 0, 10, &config()), 20.0);
    }

    #[test]
    fn test_abuse_threshold() {
        assert_eq!(calculate_abuse_threshold(&config(), false), 200.0);
        assert_eq!(calculate_abuse_threshold(&config(), true), 400.0);
    }
}

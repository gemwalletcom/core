use std::error::Error;

use gem_rewards::{IpSecurityClient, ReferralError, RewardsError, RiskScoreConfig, RiskScoringInput, evaluate_risk};
use primitives::rewards::RewardRedemptionOption;
use primitives::{ConfigKey, Localize, NaiveDateTimeExt, ReferralAllowance, ReferralLeaderboard, ReferralQuota, RewardEvent, Rewards, now};
use storage::{ConfigRepository, Database, ReferralValidationError, RewardsRedemptionsRepository, RewardsRepository, RiskSignalsRepository};
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

use crate::auth::VerifiedAuth;

enum ReferralProcessResult {
    Success(i32),
    Failed(ReferralError),
    RiskScoreExceeded(i32, ReferralError),
}

struct ReferralLimitsConfig {
    tor_allowed: bool,
    ineligible_countries: Vec<String>,
    daily_limit: i64,
    device_daily_limit: i64,
    ip_daily_limit: i64,
    ip_weekly_limit: i64,
}

pub struct RewardsClient {
    db: Database,
    stream_producer: StreamProducer,
    ip_security_client: IpSecurityClient,
}

impl RewardsClient {
    pub fn new(database: Database, stream_producer: StreamProducer, ip_security_client: IpSecurityClient) -> Self {
        Self {
            db: database,
            stream_producer,
            ip_security_client,
        }
    }

    pub fn get_rewards(&mut self, address: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let mut rewards = match self.db.rewards()?.get_reward_by_address(address) {
            Ok(r) => r,
            Err(storage::DatabaseError::NotFound) => return Ok(Rewards::default()),
            Err(e) => return Err(e.into()),
        };

        rewards.referral_allowance = self.calculate_referral_allowance(address, rewards.verified)?;
        Ok(rewards)
    }

    fn calculate_referral_allowance(&mut self, address: &str, is_verified: bool) -> Result<ReferralAllowance, Box<dyn Error + Send + Sync>> {
        let (daily_key, weekly_key) = if is_verified {
            (ConfigKey::ReferralPerVerifiedUserDaily, ConfigKey::ReferralPerVerifiedUserWeekly)
        } else {
            (ConfigKey::ReferralPerUserDaily, ConfigKey::ReferralPerUserWeekly)
        };

        let daily_limit = self.db.config()?.get_config_i64(daily_key)? as i32;
        let weekly_limit = self.db.config()?.get_config_i64(weekly_key)? as i32;

        let username = match self.db.rewards()?.get_username_by_address(address)? {
            Some(u) => u,
            None => {
                return Ok(ReferralAllowance {
                    daily: ReferralQuota {
                        limit: daily_limit,
                        available: 0,
                    },
                    weekly: ReferralQuota {
                        limit: weekly_limit,
                        available: 0,
                    },
                });
            }
        };

        let current = now();
        let daily_used = self.db.rewards()?.count_referrals_since(&username, current.days_ago(1))? as i32;
        let weekly_used = self.db.rewards()?.count_referrals_since(&username, current.days_ago(7))? as i32;

        Ok(ReferralAllowance {
            daily: ReferralQuota {
                limit: daily_limit,
                available: (daily_limit - daily_used).max(0),
            },
            weekly: ReferralQuota {
                limit: weekly_limit,
                available: (weekly_limit - weekly_used).max(0),
            },
        })
    }

    pub fn get_rewards_events(&mut self, address: &str) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
        Ok(self.db.rewards()?.get_reward_events_by_address(address)?)
    }

    pub fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, Box<dyn Error + Send + Sync>> {
        Ok(self.db.rewards()?.get_rewards_leaderboard()?)
    }

    pub fn get_rewards_redemption_option(&mut self, code: &str) -> Result<RewardRedemptionOption, Box<dyn Error + Send + Sync>> {
        Ok(self.db.rewards_redemptions()?.get_redemption_option(code)?)
    }

    pub async fn create_referral(&mut self, address: &str, code: &str, device_id: i32, ip_address: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let ip_limit = self.db.config()?.get_config_i64(ConfigKey::UsernameCreationPerIp)?;
        self.ip_security_client.check_username_creation_limit(ip_address, ip_limit).await?;

        let device_limit = self.db.config()?.get_config_i64(ConfigKey::UsernameCreationPerDevice)?;
        self.ip_security_client.check_username_creation_device_limit(device_id, device_limit).await?;

        let (rewards, event_id) = self.db.rewards()?.create_reward(address, code, device_id)?;
        self.ip_security_client.record_username_creation(ip_address).await?;
        self.ip_security_client.record_username_creation_device(device_id).await?;
        self.publish_events(vec![event_id]).await?;
        Ok(rewards)
    }

    #[allow(dead_code)]
    pub fn change_username(&mut self, address: &str, new_username: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        Ok(self.db.rewards()?.change_username(address, new_username)?)
    }

    pub async fn use_referral_code(&mut self, auth: &VerifiedAuth, code: &str, ip_address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let locale = &auth.device.locale;

        let referrer_username = self.db.rewards()?.get_referrer_username(code)?.ok_or_else(|| {
            let error = ReferralError::from(ReferralValidationError::CodeDoesNotExist);
            RewardsError::Referral(error.localize(locale))
        })?;

        match self.validate_and_score_referral(auth, &referrer_username, ip_address).await {
            ReferralProcessResult::Success(risk_signal_id) => {
                let event_ids = self
                    .db
                    .rewards()?
                    .use_or_verify_referral(&referrer_username, &auth.address, auth.device.id, risk_signal_id)?;
                self.publish_events(event_ids).await?;
                Ok(())
            }
            ReferralProcessResult::Failed(error) => {
                let _ = self
                    .db
                    .rewards()?
                    .add_referral_attempt(&referrer_username, &auth.address, auth.device.id, None, &error.to_string());
                Err(RewardsError::Referral(error.localize(locale)).into())
            }
            ReferralProcessResult::RiskScoreExceeded(risk_signal_id, error) => {
                let _ = self
                    .db
                    .rewards()?
                    .add_referral_attempt(&referrer_username, &auth.address, auth.device.id, Some(risk_signal_id), &error.to_string());
                Err(RewardsError::Referral(error.localize(locale)).into())
            }
        }
    }

    async fn validate_and_score_referral(&mut self, auth: &VerifiedAuth, referrer_username: &str, ip_address: &str) -> ReferralProcessResult {
        if let Err(e) = self.check_referrer_limits(referrer_username) {
            return ReferralProcessResult::Failed(e);
        }

        if let Err(e) = self.validate_referral_usage(auth, referrer_username) {
            return ReferralProcessResult::Failed(e);
        }

        let ip_result = match self.ip_security_client.check_ip(ip_address).await {
            Ok(result) => result,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let mut client = match self.db.client() {
            Ok(c) => c,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let limits_config = match Self::load_referral_limits_config(client.config()) {
            Ok(config) => config,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let risk_score_config = match Self::load_risk_score_config(client.config()) {
            Ok(config) => config,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let since = now().ago(risk_score_config.lookback);

        let referrer_verified = match client.rewards().is_verified_by_username(referrer_username) {
            Ok(verified) => verified,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let scoring_input = RiskScoringInput {
            username: referrer_username.to_string(),
            device_id: auth.device.id,
            device_platform: auth.device.platform.clone(),
            device_platform_store: auth.device.platform_store.clone().map(|ps| ps.to_string()).unwrap_or_default(),
            device_os: auth.device.os.clone().unwrap_or_default(),
            device_model: auth.device.model.clone().unwrap_or_default(),
            device_locale: auth.device.locale.clone(),
            device_currency: auth.device.currency.clone(),
            ip_result,
            referrer_verified,
        };

        let signal_input = scoring_input.to_signal_input();
        let fingerprint = signal_input.generate_fingerprint();

        if client.has_fingerprint_for_referrer(&fingerprint, referrer_username, since).unwrap_or(false) {
            return ReferralProcessResult::Failed(ReferralError::DuplicateAttempt);
        }

        if !limits_config.tor_allowed && scoring_input.ip_result.is_tor {
            return ReferralProcessResult::Failed(ReferralError::IpTorNotAllowed);
        }

        if limits_config.ineligible_countries.contains(&scoring_input.ip_result.country_code) {
            return ReferralProcessResult::Failed(ReferralError::IpCountryIneligible(scoring_input.ip_result.country_code.clone()));
        }

        let daily_count = match client.count_signals_since(None, now().days_ago(1)) {
            Ok(count) => count,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };
        if daily_count >= limits_config.daily_limit {
            return ReferralProcessResult::Failed(ReferralError::LimitReached(ConfigKey::ReferralUseDailyLimit));
        }

        let device_daily_count = match client.count_signals_for_device_id(scoring_input.device_id, now().days_ago(1)) {
            Ok(count) => count,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };
        if device_daily_count >= limits_config.device_daily_limit {
            return ReferralProcessResult::Failed(ReferralError::LimitReached(ConfigKey::ReferralPerDeviceDaily));
        }

        let ip_daily_count = match client.count_signals_since(Some(&scoring_input.ip_result.ip_address), now().days_ago(1)) {
            Ok(count) => count,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };
        if ip_daily_count >= limits_config.ip_daily_limit {
            return ReferralProcessResult::Failed(ReferralError::LimitReached(ConfigKey::ReferralPerIpDaily));
        }

        let ip_weekly_count = match client.count_signals_since(Some(&scoring_input.ip_result.ip_address), now().days_ago(7)) {
            Ok(count) => count,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };
        if ip_weekly_count >= limits_config.ip_weekly_limit {
            return ReferralProcessResult::Failed(ReferralError::LimitReached(ConfigKey::ReferralPerIpWeekly));
        }

        let existing_signals = match client.get_matching_risk_signals(
            &fingerprint,
            &signal_input.ip_address,
            &signal_input.ip_isp,
            &signal_input.device_model,
            signal_input.device_id,
            since,
        ) {
            Ok(signals) => signals,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let device_model_ring_count = match client.count_unique_referrers_for_device_model_pattern(
            &signal_input.device_model,
            &signal_input.device_platform,
            &signal_input.device_locale,
            since,
        ) {
            Ok(count) => count,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let risk_result = evaluate_risk(&scoring_input, &existing_signals, device_model_ring_count, &risk_score_config);
        let risk_signal_id = match client.add_risk_signal(risk_result.signal) {
            Ok(id) => id,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        if !risk_result.score.is_allowed {
            let error = ReferralError::RiskScoreExceeded {
                score: risk_result.score.score,
                max_allowed: risk_score_config.max_allowed_score,
            };
            return ReferralProcessResult::RiskScoreExceeded(risk_signal_id, error);
        }

        ReferralProcessResult::Success(risk_signal_id)
    }

    fn check_referrer_limits(&mut self, referrer_username: &str) -> Result<(), ReferralError> {
        let is_verified = self.db.rewards()?.is_verified_by_username(referrer_username)?;

        let (daily_key, weekly_key) = if is_verified {
            (ConfigKey::ReferralPerVerifiedUserDaily, ConfigKey::ReferralPerVerifiedUserWeekly)
        } else {
            (ConfigKey::ReferralPerUserDaily, ConfigKey::ReferralPerUserWeekly)
        };

        let current = now();

        let daily_limit = self.db.config()?.get_config_i64(daily_key)?;
        if self.db.rewards()?.count_referrals_since(referrer_username, current.days_ago(1))? >= daily_limit {
            return Err(ReferralError::ReferrerLimitReached("daily".to_string()));
        }

        let weekly_limit = self.db.config()?.get_config_i64(weekly_key)?;
        if self.db.rewards()?.count_referrals_since(referrer_username, current.days_ago(7))? >= weekly_limit {
            return Err(ReferralError::ReferrerLimitReached("weekly".to_string()));
        }

        Ok(())
    }

    fn validate_referral_usage(&mut self, auth: &VerifiedAuth, referrer_username: &str) -> Result<(), ReferralError> {
        let eligibility = self.db.config()?.get_config_duration(ConfigKey::ReferralEligibility)?;
        let eligibility_days = (eligibility.as_secs() / 86400) as i64;
        let eligibility_cutoff = now().ago(eligibility);

        self.db
            .rewards()?
            .validate_referral_use(referrer_username, auth.device.id, eligibility_days)?;

        let first_subscription_date = self
            .db
            .rewards()?
            .get_first_subscription_date(vec![auth.address.clone()])?;

        let is_new_device = auth.device.created_at > eligibility_cutoff;
        let is_new_subscription = first_subscription_date.map(|d| d > eligibility_cutoff).unwrap_or(true);

        if !is_new_device || !is_new_subscription {
            return Err(ReferralValidationError::EligibilityExpired(eligibility_days).into());
        }

        Ok(())
    }

    fn load_referral_limits_config(config: &mut dyn storage::ConfigRepository) -> Result<ReferralLimitsConfig, storage::DatabaseError> {
        Ok(ReferralLimitsConfig {
            tor_allowed: config.get_config_bool(ConfigKey::ReferralIpTorAllowed)?,
            ineligible_countries: config.get_config_vec_string(ConfigKey::ReferralIneligibleCountries)?,
            daily_limit: config.get_config_i64(ConfigKey::ReferralUseDailyLimit)?,
            device_daily_limit: config.get_config_i64(ConfigKey::ReferralPerDeviceDaily)?,
            ip_daily_limit: config.get_config_i64(ConfigKey::ReferralPerIpDaily)?,
            ip_weekly_limit: config.get_config_i64(ConfigKey::ReferralPerIpWeekly)?,
        })
    }

    fn load_risk_score_config(config: &mut dyn storage::ConfigRepository) -> Result<RiskScoreConfig, storage::DatabaseError> {
        Ok(RiskScoreConfig {
            fingerprint_match_penalty_per_referrer: config.get_config_i64(ConfigKey::ReferralRiskScoreFingerprintMatchPerReferrer)?,
            fingerprint_match_max_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreFingerprintMatchMaxPenalty)?,
            ip_reuse_score: config.get_config_i64(ConfigKey::ReferralRiskScoreIpReuse)?,
            isp_model_match_score: config.get_config_i64(ConfigKey::ReferralRiskScoreIspModelMatch)?,
            device_id_reuse_penalty_per_referrer: config.get_config_i64(ConfigKey::ReferralRiskScoreDeviceIdReusePerReferrer)?,
            device_id_reuse_max_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreDeviceIdReuseMaxPenalty)?,
            ineligible_ip_type_score: config.get_config_i64(ConfigKey::ReferralRiskScoreIneligibleIpType)?,
            blocked_ip_types: config.get_config_vec_string(ConfigKey::ReferralBlockedIpTypes)?,
            blocked_ip_type_penalty: config.get_config_i64(ConfigKey::ReferralBlockedIpTypePenalty)?,
            max_abuse_score: config.get_config_i64(ConfigKey::ReferralMaxAbuseScore)?,
            penalty_isps: config.get_config_vec_string(ConfigKey::ReferralPenaltyIsps)?,
            isp_penalty_score: config.get_config_i64(ConfigKey::ReferralPenaltyIspsScore)?,
            verified_user_reduction: config.get_config_i64(ConfigKey::ReferralRiskScoreVerifiedUserReduction)?,
            max_allowed_score: config.get_config_i64(ConfigKey::ReferralRiskScoreMaxAllowed)?,
            same_referrer_pattern_threshold: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternThreshold)?,
            same_referrer_pattern_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternPenalty)?,
            same_referrer_fingerprint_threshold: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintThreshold)?,
            same_referrer_fingerprint_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintPenalty)?,
            same_referrer_device_model_threshold: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerDeviceModelThreshold)?,
            same_referrer_device_model_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerDeviceModelPenalty)?,
            device_model_ring_threshold: config.get_config_i64(ConfigKey::ReferralRiskScoreDeviceModelRingThreshold)?,
            device_model_ring_penalty_per_member: config.get_config_i64(ConfigKey::ReferralRiskScoreDeviceModelRingPenaltyPerMember)?,
            lookback: config.get_config_duration(ConfigKey::ReferralRiskScoreLookback)?,
            high_risk_platform_stores: config.get_config_vec_string(ConfigKey::ReferralRiskScoreHighRiskPlatformStores)?,
            high_risk_platform_store_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreHighRiskPlatformStorePenalty)?,
            high_risk_countries: config.get_config_vec_string(ConfigKey::ReferralRiskScoreHighRiskCountries)?,
            high_risk_country_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreHighRiskCountryPenalty)?,
            high_risk_locales: config.get_config_vec_string(ConfigKey::ReferralRiskScoreHighRiskLocales)?,
            high_risk_locale_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreHighRiskLocalePenalty)?,
        })
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer
            .publish_rewards_events(event_ids.into_iter().map(RewardsNotificationPayload::new).collect())
            .await?;
        Ok(())
    }
}

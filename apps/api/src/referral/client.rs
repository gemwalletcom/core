use std::error::Error;

use api_connector::PusherClient;
use gem_rewards::{IpSecurityClient, ReferralError, RewardsError, RiskScoreConfig, RiskScoringInput, UsernameError, evaluate_risk};
use primitives::rewards::RewardRedemptionOption;
use primitives::{ConfigKey, IpUsageType, Localize, NaiveDateTimeExt, Platform, ReferralAllowance, ReferralLeaderboard, ReferralQuota, RewardEvent, Rewards, WalletId, now};
use storage::{
    ConfigCacher, Database, NewWalletRow, ReferralValidationError, RewardsRedemptionsRepository, RewardsRepository, RiskSignalsRepository, WalletSource, WalletType,
    WalletsRepository,
};
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
    country_daily_limit: i64,
}

pub struct RewardsClient {
    db: Database,
    config: ConfigCacher,
    stream_producer: StreamProducer,
    ip_security_client: IpSecurityClient,
    pusher: PusherClient,
}

impl RewardsClient {
    pub fn new(database: Database, stream_producer: StreamProducer, ip_security_client: IpSecurityClient, pusher: PusherClient) -> Self {
        let config = ConfigCacher::new(database.clone());
        Self {
            db: database,
            config,
            stream_producer,
            ip_security_client,
            pusher,
        }
    }

    fn map_username_error(&self, error: Box<dyn Error + Send + Sync>, locale: &str) -> RewardsError {
        if let Some(username_error) = error.downcast_ref::<UsernameError>() {
            RewardsError::Username(username_error.localize(locale))
        } else {
            RewardsError::Username(error.to_string())
        }
    }

    pub fn get_rewards(&self, wallet_identifier: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let wallet = match self.db.wallets()?.get_wallet(wallet_identifier) {
            Ok(w) => w,
            Err(storage::DatabaseError::NotFound) => return Ok(Rewards::default()),
            Err(e) => return Err(e.into()),
        };

        let mut rewards = match self.db.rewards()?.get_reward_by_wallet_id(wallet.id) {
            Ok(r) => r,
            Err(storage::DatabaseError::NotFound) => return Ok(Rewards::default()),
            Err(e) => return Err(e.into()),
        };

        rewards.referral_allowance = self.calculate_referral_allowance(wallet.id, rewards.status.is_verified())?;
        Ok(rewards)
    }

    fn calculate_referral_allowance(&self, wallet_id: i32, is_verified: bool) -> Result<ReferralAllowance, Box<dyn Error + Send + Sync>> {
        let multiplier = if is_verified { self.config.get_i32(ConfigKey::ReferralVerifiedMultiplier)? } else { 1 };

        let daily_limit = self.config.get_i32(ConfigKey::ReferralPerUserDaily)? * multiplier;
        let weekly_limit = self.config.get_i32(ConfigKey::ReferralPerUserWeekly)? * multiplier;

        let username = match self.db.rewards()?.get_username_by_wallet_id(wallet_id)? {
            Some(u) => u,
            None => {
                return Ok(ReferralAllowance {
                    daily: ReferralQuota { limit: daily_limit, available: 0 },
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

    pub fn get_rewards_events(&self, wallet_identifier: &str) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
        let wallet = self.db.wallets()?.get_wallet(wallet_identifier)?;
        Ok(self.db.rewards()?.get_reward_events_by_wallet_id(wallet.id)?)
    }

    pub fn get_rewards_leaderboard(&self) -> Result<ReferralLeaderboard, Box<dyn Error + Send + Sync>> {
        Ok(self.db.rewards()?.get_rewards_leaderboard()?)
    }

    pub fn get_rewards_redemption_option(&self, code: &str) -> Result<RewardRedemptionOption, Box<dyn Error + Send + Sync>> {
        Ok(self.db.rewards_redemptions()?.get_redemption_option(code)?)
    }

    pub async fn create_username(&self, wallet_identifier: &str, code: &str, device_id: i32, ip_address: &str, locale: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let wallet = self.db.wallets()?.get_wallet(wallet_identifier)?;

        let global_daily_limit = self.config.get_i64(ConfigKey::UsernameCreationGlobalDailyLimit)?;
        let ip_limit = self.config.get_i64(ConfigKey::UsernameCreationPerIp)?;
        let device_limit = self.config.get_i64(ConfigKey::UsernameCreationPerDevice)?;
        let country_daily_limit = self.config.get_i64(ConfigKey::UsernameCreationPerCountryDailyLimit)?;
        let ineligible_countries = self.config.get_vec_string(ConfigKey::ReferralIneligibleCountries)?;
        let blocked_ip_types: Vec<IpUsageType> = self.config.get_vec(ConfigKey::ReferralBlockedIpTypes)?;

        self.ip_security_client
            .check_username_creation_limits(ip_address, device_id, global_daily_limit, ip_limit, device_limit)
            .await
            .map_err(|e| self.map_username_error(e, locale))?;

        let ip_result = self.ip_security_client.check_ip(ip_address).await?;

        self.ip_security_client
            .check_username_creation_country_limit(&ip_result.country_code, country_daily_limit)
            .await
            .map_err(|e| self.map_username_error(e, locale))?;

        self.ip_security_client
            .check_username_creation_country_eligibility(&ip_result.country_code, &ineligible_countries)
            .map_err(|e| self.map_username_error(e, locale))?;

        self.ip_security_client
            .check_username_creation_ip_type(ip_result.usage_type, &blocked_ip_types)
            .map_err(|e| self.map_username_error(e, locale))?;

        let (rewards, event_id) = self.db.rewards()?.create_reward(wallet.id, code, device_id)?;
        self.ip_security_client.record_username_creation(&ip_result.country_code, ip_address, device_id).await?;
        self.publish_events(vec![event_id]).await?;
        Ok(rewards)
    }

    #[allow(dead_code)]
    pub fn change_username(&self, wallet_identifier: &str, new_username: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let wallet = self.db.wallets()?.get_wallet(wallet_identifier)?;
        Ok(self.db.rewards()?.change_username(wallet.id, new_username)?)
    }

    pub async fn use_referral_code(&self, auth: &VerifiedAuth, code: &str, ip_address: &str) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
        let locale = auth.device.locale.as_str();
        let wallet_identifier = WalletId::Multicoin(auth.address.clone()).id();
        let wallet = self.db.wallets()?.get_or_create_wallet(NewWalletRow {
            identifier: wallet_identifier,
            wallet_type: WalletType::Multicoin,
            source: WalletSource::Import,
        })?;

        let referrer_username = self.db.rewards()?.get_referral_code(code)?.ok_or_else(|| {
            let error = ReferralError::from(ReferralValidationError::CodeDoesNotExist);
            RewardsError::Referral(error.localize(locale))
        })?;

        match self.validate_and_score_referral(auth, &referrer_username, ip_address).await {
            ReferralProcessResult::Success(risk_signal_id) => {
                let events = self.db.rewards()?.use_or_verify_referral(&referrer_username, wallet.id, auth.device.id, risk_signal_id)?;
                Ok(events)
            }
            ReferralProcessResult::Failed(error) => {
                let _ = self
                    .db
                    .rewards()?
                    .add_referral_attempt(&referrer_username, wallet.id, auth.device.id, None, &error.to_string());
                Err(RewardsError::Referral(error.localize(locale)).into())
            }
            ReferralProcessResult::RiskScoreExceeded(risk_signal_id, error) => {
                let _ = self
                    .db
                    .rewards()?
                    .add_referral_attempt(&referrer_username, wallet.id, auth.device.id, Some(risk_signal_id), &error.to_string());
                Err(RewardsError::Referral(error.localize(locale)).into())
            }
        }
    }

    async fn validate_and_score_referral(&self, auth: &VerifiedAuth, referrer_username: &str, ip_address: &str) -> ReferralProcessResult {
        if let Err(e) = self.check_referrer_limits(referrer_username) {
            return ReferralProcessResult::Failed(e);
        }

        if let Err(e) = self.validate_referral_usage(auth, referrer_username) {
            return ReferralProcessResult::Failed(e);
        }

        if *auth.device.platform == Platform::Android {
            match self.pusher.is_device_token_valid(&auth.device.token, auth.device.platform.as_i32()).await {
                Ok(true) => {}
                Ok(false) => return ReferralProcessResult::Failed(ReferralError::InvalidDeviceToken("token_not_registered".to_string())),
                Err(e) => return ReferralProcessResult::Failed(ReferralError::InvalidDeviceToken(e.to_string())),
            }
        }

        let ip_result = match self.ip_security_client.check_ip(ip_address).await {
            Ok(result) => result,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let mut client = match self.db.client() {
            Ok(c) => c,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let limits_config = match self.load_referral_limits_config() {
            Ok(config) => config,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let risk_score_config = match self.load_risk_score_config() {
            Ok(config) => config,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let since = now().ago(risk_score_config.lookback);

        let banned_user_count = match client.count_disabled_users_by_device(auth.device.id, since) {
            Ok(count) => count,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };
        if banned_user_count > 0 {
            return ReferralProcessResult::Failed(ReferralError::LimitReached(ConfigKey::ReferralPerDeviceDaily));
        }

        let referrer_verified = match client.rewards().is_verified_by_username(referrer_username) {
            Ok(verified) => verified,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let scoring_input = RiskScoringInput {
            username: referrer_username.to_string(),
            device_id: auth.device.id,
            device_platform: *auth.device.platform,
            device_platform_store: *auth.device.platform_store,
            device_os: auth.device.os.clone(),
            device_model: auth.device.model.clone(),
            device_locale: auth.device.locale.as_str().to_string(),
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

        let country_daily_count = match client.count_signals_for_country(&scoring_input.ip_result.country_code, now().days_ago(1)) {
            Ok(count) => count,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };
        if country_daily_count >= limits_config.country_daily_limit {
            return ReferralProcessResult::Failed(ReferralError::LimitReached(ConfigKey::ReferralPerCountryDaily));
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

        let device_model_ring_count =
            match client.count_unique_referrers_for_device_model_pattern(&signal_input.device_model, signal_input.device_platform, &signal_input.device_locale, since) {
                Ok(count) => count,
                Err(e) => return ReferralProcessResult::Failed(e.into()),
            };

        let ip_abuser_count = match client.count_disabled_users_by_ip(&signal_input.ip_address, since) {
            Ok(count) => count,
            Err(e) => return ReferralProcessResult::Failed(e.into()),
        };

        let risk_result = evaluate_risk(&scoring_input, &existing_signals, device_model_ring_count, ip_abuser_count, &risk_score_config);
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

    fn check_referrer_limits(&self, referrer_username: &str) -> Result<(), ReferralError> {
        let is_verified = self.db.rewards()?.is_verified_by_username(referrer_username)?;
        let multiplier = if is_verified { self.config.get_i64(ConfigKey::ReferralVerifiedMultiplier)? } else { 1 };

        let current = now();

        let cooldown = self.config.get_duration(ConfigKey::ReferralCooldown)?;
        if self.db.rewards()?.count_referrals_since(referrer_username, current.ago(cooldown))? >= 1 {
            return Err(ReferralError::ReferrerLimitReached(ConfigKey::ReferralCooldown));
        }

        let hourly_limit = self.config.get_i64(ConfigKey::ReferralPerUserHourly)? * multiplier;
        if self.db.rewards()?.count_referrals_since(referrer_username, current.hours_ago(1))? >= hourly_limit {
            return Err(ReferralError::ReferrerLimitReached(ConfigKey::ReferralPerUserHourly));
        }

        let daily_limit = self.config.get_i64(ConfigKey::ReferralPerUserDaily)? * multiplier;
        if self.db.rewards()?.count_referrals_since(referrer_username, current.days_ago(1))? >= daily_limit {
            return Err(ReferralError::ReferrerLimitReached(ConfigKey::ReferralPerUserDaily));
        }

        let weekly_limit = self.config.get_i64(ConfigKey::ReferralPerUserWeekly)? * multiplier;
        if self.db.rewards()?.count_referrals_since(referrer_username, current.days_ago(7))? >= weekly_limit {
            return Err(ReferralError::ReferrerLimitReached(ConfigKey::ReferralPerUserWeekly));
        }

        Ok(())
    }

    fn validate_referral_usage(&self, auth: &VerifiedAuth, referrer_username: &str) -> Result<(), ReferralError> {
        let eligibility = self.config.get_duration(ConfigKey::ReferralEligibility)?;
        let eligibility_days = (eligibility.as_secs() / 86400) as i64;
        let eligibility_cutoff = now().ago(eligibility);

        self.db.rewards()?.validate_referral_use(referrer_username, auth.device.id, eligibility_days)?;

        let first_subscription_date = self.db.rewards()?.get_first_subscription_date(vec![auth.address.clone()])?;

        let is_new_device = auth.device.created_at > eligibility_cutoff;
        let is_new_subscription = first_subscription_date.map(|d| d > eligibility_cutoff).unwrap_or(true);

        if !is_new_device || !is_new_subscription {
            return Err(ReferralValidationError::EligibilityExpired(eligibility_days).into());
        }

        Ok(())
    }

    fn load_referral_limits_config(&self) -> Result<ReferralLimitsConfig, storage::DatabaseError> {
        Ok(ReferralLimitsConfig {
            tor_allowed: self.config.get_bool(ConfigKey::ReferralIpTorAllowed)?,
            ineligible_countries: self.config.get_vec_string(ConfigKey::ReferralIneligibleCountries)?,
            daily_limit: self.config.get_i64(ConfigKey::ReferralUseDailyLimit)?,
            device_daily_limit: self.config.get_i64(ConfigKey::ReferralPerDeviceDaily)?,
            ip_daily_limit: self.config.get_i64(ConfigKey::ReferralPerIpDaily)?,
            ip_weekly_limit: self.config.get_i64(ConfigKey::ReferralPerIpWeekly)?,
            country_daily_limit: self.config.get_i64(ConfigKey::ReferralPerCountryDaily)?,
        })
    }

    fn load_risk_score_config(&self) -> Result<RiskScoreConfig, storage::DatabaseError> {
        Ok(RiskScoreConfig {
            fingerprint_match_penalty_per_referrer: self.config.get_i64(ConfigKey::ReferralRiskScoreFingerprintMatchPerReferrer)?,
            fingerprint_match_max_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreFingerprintMatchMaxPenalty)?,
            ip_reuse_score: self.config.get_i64(ConfigKey::ReferralRiskScoreIpReuse)?,
            isp_model_match_score: self.config.get_i64(ConfigKey::ReferralRiskScoreIspModelMatch)?,
            device_id_reuse_penalty_per_referrer: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceIdReusePerReferrer)?,
            device_id_reuse_max_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceIdReuseMaxPenalty)?,
            ineligible_ip_type_score: self.config.get_i64(ConfigKey::ReferralRiskScoreIneligibleIpType)?,
            blocked_ip_types: self.config.get_vec(ConfigKey::ReferralBlockedIpTypes)?,
            blocked_ip_type_penalty: self.config.get_i64(ConfigKey::ReferralBlockedIpTypePenalty)?,
            max_abuse_score: self.config.get_i64(ConfigKey::ReferralMaxAbuseScore)?,
            penalty_isps: self.config.get_vec_string(ConfigKey::ReferralPenaltyIsps)?,
            isp_penalty_score: self.config.get_i64(ConfigKey::ReferralPenaltyIspsScore)?,
            verified_user_reduction: self.config.get_i64(ConfigKey::ReferralRiskScoreVerifiedUserReduction)?,
            max_allowed_score: self.config.get_i64(ConfigKey::ReferralRiskScoreMaxAllowed)?,
            same_referrer_pattern_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternThreshold)?,
            same_referrer_pattern_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternPenalty)?,
            same_referrer_fingerprint_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintThreshold)?,
            same_referrer_fingerprint_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintPenalty)?,
            same_referrer_device_model_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerDeviceModelThreshold)?,
            same_referrer_device_model_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerDeviceModelPenalty)?,
            device_model_ring_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceModelRingThreshold)?,
            device_model_ring_penalty_per_member: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceModelRingPenaltyPerMember)?,
            lookback: self.config.get_duration(ConfigKey::ReferralRiskScoreLookback)?,
            high_risk_platform_stores: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskPlatformStores)?,
            high_risk_platform_store_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskPlatformStorePenalty)?,
            high_risk_countries: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskCountries)?,
            high_risk_country_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskCountryPenalty)?,
            high_risk_locales: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskLocales)?,
            high_risk_locale_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskLocalePenalty)?,
            high_risk_device_models: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskDeviceModels)?,
            high_risk_device_model_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskDeviceModelPenalty)?,
            ip_history_penalty_per_abuser: self.config.get_i64(ConfigKey::ReferralRiskScoreIpHistoryPenaltyPerAbuser)?,
            ip_history_max_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreIpHistoryMaxPenalty)?,
            velocity_window: self.config.get_duration(ConfigKey::ReferralAbuseVelocityWindow)?,
            velocity_divisor: self.config.get_i64(ConfigKey::ReferralAbuseVelocityDivisor)?,
            velocity_penalty: self.config.get_i64(ConfigKey::ReferralAbuseVelocityPenaltyPerSignal)?,
            referral_per_user_daily: self.config.get_i64(ConfigKey::ReferralPerUserDaily)?,
            verified_multiplier: self.config.get_i64(ConfigKey::ReferralVerifiedMultiplier)?,
            cross_referrer_device_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerDevicePenalty)?,
        })
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer
            .publish_rewards_events(event_ids.into_iter().map(RewardsNotificationPayload::new).collect())
            .await?;
        Ok(())
    }
}

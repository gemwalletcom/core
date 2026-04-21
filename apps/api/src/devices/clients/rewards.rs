use std::error::Error;

use api_connector::PusherClient;
use chrono::NaiveDateTime;
use gem_rewards::{IpSecurityClient, ReferralError, RewardsError, RiskScoreConfig, RiskScoringInput, UsernameError, evaluate_risk};
use primitives::rewards::{RewardRedemptionOption, RewardStatus};
use primitives::{ConfigKey, Localize, NaiveDateTimeExt, Platform, ReferralLeaderboard, RewardEvent, Rewards, WalletId, now};
use storage::models::DeviceRow;
use storage::{
    ConfigCacher, Database, NewWalletRow, ReferralValidationError, RewardsRedemptionsRepository, RewardsRepository, RiskSignalsRepository, WalletSource, WalletType,
    WalletsRepository,
};
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

enum ReferralProcessResult {
    Success { risk_signal_id: i32, referrer_status: RewardStatus },
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

fn referrer_multiplier(config: &ConfigCacher, status: &RewardStatus) -> Result<i64, storage::DatabaseError> {
    if *status == RewardStatus::Trusted {
        config.get_i64(ConfigKey::ReferralTrustedMultiplier)
    } else {
        config.get_i64(ConfigKey::ReferralVerifiedMultiplier)
    }
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

    pub fn get_rewards_by_wallet_id(&self, wallet_id: i32) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        match self.db.rewards()?.get_reward_by_wallet_id(wallet_id) {
            Ok(r) => Ok(r),
            Err(error) if error.is_not_found() => Ok(Rewards::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_rewards_events_by_wallet_id(&self, wallet_id: i32) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
        Ok(self.db.rewards()?.get_reward_events_by_wallet_id(wallet_id)?)
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

        self.ip_security_client
            .check_username_creation_limits(ip_address, device_id, global_daily_limit, ip_limit, device_limit)
            .await
            .map_err(|e| self.map_username_error(e, locale))?;

        let ip_result = self.ip_security_client.check_ip(ip_address).await?;

        self.ip_security_client
            .check_username_creation_country_limit(&ip_result.country_code, country_daily_limit)
            .await
            .map_err(|e| self.map_username_error(e, locale))?;

        let (rewards, event_id) = self
            .db
            .rewards()?
            .create_reward(wallet.id, code)
            .map_err(|e| RewardsError::Username(UsernameError::Validation(e).localize(locale)))?;
        self.ip_security_client.record_username_creation(&ip_result.country_code, ip_address, device_id).await?;
        self.publish_events(vec![event_id]).await?;
        Ok(rewards)
    }

    pub async fn use_referral_code(
        &self,
        device: &DeviceRow,
        address: &str,
        code: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
        let locale = device.locale.as_str();
        let wallet_identifier = WalletId::Multicoin(address.to_string()).id();
        let wallet = self.db.wallets()?.get_or_create_wallet(NewWalletRow {
            identifier: wallet_identifier,
            wallet_type: WalletType::Multicoin,
            source: WalletSource::Import,
        })?;

        let mut client = self.db.client()?;

        let referrer_username = client.rewards().get_referral_code(code)?.ok_or_else(|| {
            let error = ReferralError::from(ReferralValidationError::CodeDoesNotExist);
            RewardsError::Referral(error.localize(locale))
        })?;

        if client.rewards().is_pending_referral(&referrer_username, wallet.id, device.id)? {
            let referrer_info = client.rewards().get_referrer_info(&referrer_username)?;
            if !referrer_info.status.is_verified() {
                return Err(RewardsError::Referral(ReferralError::from(ReferralValidationError::RewardsNotEnabled(referrer_username.clone())).localize(locale)).into());
            }
            let events = client
                .rewards()
                .use_or_verify_referral(&referrer_username, &referrer_info.status, wallet.id, device.id, 0)?;
            return Ok(events);
        }
        drop(client);

        match self.validate_and_score_referral(device, wallet.id, &referrer_username, ip_address, user_agent).await {
            ReferralProcessResult::Success { risk_signal_id, referrer_status } => {
                let events = self
                    .db
                    .rewards()?
                    .use_or_verify_referral(&referrer_username, &referrer_status, wallet.id, device.id, risk_signal_id)?;
                Ok(events)
            }
            ReferralProcessResult::Failed(error) => {
                let _ = self.db.rewards()?.add_referral_attempt(&referrer_username, wallet.id, device.id, None, &error.to_string());
                Err(RewardsError::Referral(error.localize(locale)).into())
            }
            ReferralProcessResult::RiskScoreExceeded(risk_signal_id, error) => {
                let _ = self
                    .db
                    .rewards()?
                    .add_referral_attempt(&referrer_username, wallet.id, device.id, Some(risk_signal_id), &error.to_string());
                Err(RewardsError::Referral(error.localize(locale)).into())
            }
        }
    }

    async fn validate_and_score_referral(&self, device: &DeviceRow, wallet_id: i32, referrer_username: &str, ip_address: &str, user_agent: &str) -> ReferralProcessResult {
        match self.validate_and_score_referral_inner(device, wallet_id, referrer_username, ip_address, user_agent).await {
            Ok(result) => result,
            Err(e) => ReferralProcessResult::Failed(e),
        }
    }

    async fn validate_and_score_referral_inner(
        &self,
        device: &DeviceRow,
        wallet_id: i32,
        referrer_username: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<ReferralProcessResult, ReferralError> {
        let referrer_info;
        {
            let mut client = self.db.client()?;

            referrer_info = client.rewards().get_referrer_info(referrer_username)?;
            if !referrer_info.status.is_verified() {
                return Err(ReferralValidationError::RewardsNotEnabled(referrer_username.to_string()).into());
            }

            let multiplier = referrer_multiplier(&self.config, &referrer_info.status)?;
            let current = now();
            let cooldown = self.config.get_duration(ConfigKey::ReferralCooldown)?;

            self.check_referrer_rate_limit(&mut client, referrer_username, current.days_ago(7), ConfigKey::ReferralPerUserWeekly, multiplier)?;
            self.check_referrer_rate_limit(&mut client, referrer_username, current.days_ago(1), ConfigKey::ReferralPerUserDaily, multiplier)?;
            self.check_referrer_rate_limit(&mut client, referrer_username, current.hours_ago(1), ConfigKey::ReferralPerUserHourly, multiplier)?;

            if client.rewards().count_referrals_since(referrer_username, current.ago(cooldown))? >= 1 {
                return Err(ReferralError::ReferrerLimitReached(ConfigKey::ReferralCooldown));
            }

            let eligibility = self.config.get_duration(ConfigKey::ReferralEligibility)?;
            let eligibility_days = (eligibility.as_secs() / 86400) as i64;
            client
                .rewards()
                .validate_referral_use(referrer_username, referrer_info.wallet_id, wallet_id, device.id, device.created_at, eligibility_days)?;
        }

        if *device.platform == Platform::Android {
            match self.pusher.is_device_token_valid(&device.token, device.platform.as_i32()).await {
                Ok(true) => {}
                Ok(false) => return Err(ReferralError::InvalidDeviceToken("token_not_registered".to_string())),
                Err(e) => return Err(ReferralError::InvalidDeviceToken(e.to_string())),
            }
        }

        let ip_result = self.ip_security_client.check_ip(ip_address).await?;
        let limits_config = self.load_referral_limits_config()?;
        let risk_score_config = self.load_risk_score_config()?;
        let since = now().ago(risk_score_config.lookback);

        let mut client = self.db.client()?;

        if client.count_disabled_users_by_device(device.id, since)? > 0 {
            return Err(ReferralError::LimitReached(ConfigKey::ReferralPerDeviceDaily));
        }

        let scoring_input = RiskScoringInput {
            username: referrer_username.to_string(),
            device_id: device.id,
            device_platform: *device.platform,
            device_platform_store: *device.platform_store,
            device_os: device.os.clone(),
            device_model: device.model.clone(),
            device_locale: device.locale.as_str().to_string(),
            device_currency: device.currency.clone(),
            ip_result,
            referrer_status: referrer_info.status,
            referrer_referral_count: referrer_info.referral_count as i64,
            user_agent: user_agent.to_string(),
        };

        let signal_input = scoring_input.to_signal_input();
        let fingerprint = signal_input.generate_fingerprint();

        if client.has_fingerprint_for_referrer(&fingerprint, referrer_username, since).unwrap_or(false) {
            return Err(ReferralError::DuplicateAttempt);
        }

        if !limits_config.tor_allowed && scoring_input.ip_result.is_tor {
            return Err(ReferralError::IpTorNotAllowed);
        }

        if limits_config.ineligible_countries.contains(&scoring_input.ip_result.country_code) {
            return Err(ReferralError::IpCountryIneligible(scoring_input.ip_result.country_code.clone()));
        }

        let one_day_ago = now().days_ago(1);
        self.check_global_rate_limit(&mut client, None, one_day_ago, limits_config.daily_limit, ConfigKey::ReferralUseDailyLimit)?;
        self.check_device_rate_limit(&mut client, scoring_input.device_id, one_day_ago, limits_config.device_daily_limit)?;
        self.check_global_rate_limit(
            &mut client,
            Some(&scoring_input.ip_result.ip_address),
            one_day_ago,
            limits_config.ip_daily_limit,
            ConfigKey::ReferralPerIpDaily,
        )?;
        self.check_global_rate_limit(
            &mut client,
            Some(&scoring_input.ip_result.ip_address),
            now().days_ago(7),
            limits_config.ip_weekly_limit,
            ConfigKey::ReferralPerIpWeekly,
        )?;
        self.check_country_rate_limit(&mut client, &scoring_input.ip_result.country_code, one_day_ago, limits_config.country_daily_limit)?;

        let existing_signals = client.get_matching_risk_signals(
            &fingerprint,
            &signal_input.ip_address,
            &signal_input.ip_isp,
            &signal_input.device_model,
            signal_input.device_id,
            since,
        )?;

        let device_model_ring_count =
            client.count_unique_referrers_for_device_model_pattern(&signal_input.device_model, signal_input.device_platform, &signal_input.device_locale, since)?;
        let ip_abuser_count = client.count_disabled_users_by_ip(&signal_input.ip_address, since)?;
        let cross_referrer_fingerprint_count = client.count_unique_referrers_for_fingerprint(&fingerprint, since)?;
        let referrer_country_count = client.count_unique_countries_for_referrer(referrer_username, since)?;
        let referrer_device_count = client.count_unique_devices_for_referrer(referrer_username, since)?;

        let risk_result = evaluate_risk(
            &scoring_input,
            &existing_signals,
            device_model_ring_count,
            ip_abuser_count,
            cross_referrer_fingerprint_count,
            referrer_country_count,
            referrer_device_count,
            &risk_score_config,
        );
        let risk_signal_id = client.add_risk_signal(risk_result.signal)?;

        if !risk_result.score.is_allowed {
            return Ok(ReferralProcessResult::RiskScoreExceeded(
                risk_signal_id,
                ReferralError::RiskScoreExceeded {
                    score: risk_result.score.score,
                    max_allowed: risk_score_config.max_allowed_score,
                },
            ));
        }

        Ok(ReferralProcessResult::Success {
            risk_signal_id,
            referrer_status: referrer_info.status,
        })
    }

    fn check_referrer_rate_limit(
        &self,
        client: &mut storage::DatabaseClient,
        referrer_username: &str,
        since: NaiveDateTime,
        key: ConfigKey,
        multiplier: i64,
    ) -> Result<(), ReferralError> {
        let count = client.rewards().count_referrals_since(referrer_username, since)?;
        let limit = self.config.get_i64(key.clone())? * multiplier;
        if count >= limit {
            return Err(ReferralError::ReferrerLimitReached(key));
        }
        Ok(())
    }

    fn check_global_rate_limit(
        &self,
        client: &mut storage::DatabaseClient,
        ip_address: Option<&str>,
        since: NaiveDateTime,
        limit: i64,
        key: ConfigKey,
    ) -> Result<(), ReferralError> {
        if client.count_signals_since(ip_address, since)? >= limit {
            return Err(ReferralError::LimitReached(key));
        }
        Ok(())
    }

    fn check_device_rate_limit(&self, client: &mut storage::DatabaseClient, device_id: i32, since: NaiveDateTime, limit: i64) -> Result<(), ReferralError> {
        if client.count_signals_for_device_id(device_id, since)? >= limit {
            return Err(ReferralError::LimitReached(ConfigKey::ReferralPerDeviceDaily));
        }
        Ok(())
    }

    fn check_country_rate_limit(&self, client: &mut storage::DatabaseClient, country_code: &str, since: NaiveDateTime, limit: i64) -> Result<(), ReferralError> {
        if client.count_signals_for_country(country_code, since)? >= limit {
            return Err(ReferralError::LimitReached(ConfigKey::ReferralPerCountryDaily));
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
            early_referral_reduction_initial: self.config.get_i64(ConfigKey::ReferralRiskScoreEarlyReferralReductionInitial)?,
            early_referral_reduction_step: self.config.get_i64(ConfigKey::ReferralRiskScoreEarlyReferralReductionStep)?,
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
            high_risk_user_agents: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskUserAgents)?,
            high_risk_user_agent_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskUserAgentPenalty)?,
            ip_history_penalty_per_abuser: self.config.get_i64(ConfigKey::ReferralRiskScoreIpHistoryPenaltyPerAbuser)?,
            ip_history_max_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreIpHistoryMaxPenalty)?,
            velocity_window: self.config.get_duration(ConfigKey::ReferralAbuseVelocityWindow)?,
            velocity_divisor: self.config.get_i64(ConfigKey::ReferralAbuseVelocityDivisor)?,
            velocity_penalty: self.config.get_i64(ConfigKey::ReferralAbuseVelocityPenaltyPerSignal)?,
            referral_per_user_daily: self.config.get_i64(ConfigKey::ReferralPerUserDaily)?,
            verified_multiplier: self.config.get_i64(ConfigKey::ReferralVerifiedMultiplier)?,
            trusted_multiplier: self.config.get_i64(ConfigKey::ReferralTrustedMultiplier)?,
            cross_referrer_device_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerDevicePenalty)?,
            cross_referrer_fingerprint_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerFingerprintThreshold)?,
            cross_referrer_fingerprint_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerFingerprintPenalty)?,
            country_diversity_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreCountryDiversityThreshold)?,
            country_diversity_penalty_per_country: self.config.get_i64(ConfigKey::ReferralRiskScoreCountryDiversityPenaltyPerCountry)?,
            device_farming_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceFarmingThreshold)?,
            device_farming_penalty_per_device: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceFarmingPenaltyPerDevice)?,
        })
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer
            .publish_rewards_events(event_ids.into_iter().map(RewardsNotificationPayload::new).collect())
            .await?;
        Ok(())
    }
}

use std::error::Error;

use gem_rewards::{IpSecurityClient, ReferralError, RewardsError, RiskScoreConfig, RiskScoringInput, evaluate_risk};
use primitives::rewards::RewardRedemptionOption;
use primitives::{ConfigKey, NaiveDateTimeExt, ReferralLeaderboard, RewardEvent, RewardEventType, Rewards, now};
use storage::{Database, ReferralValidationError, RiskSignalsRepository};
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

use crate::auth::VerifiedAuth;

const REFERRAL_ELIGIBILITY_DAYS: i64 = 7;

struct ReferralLimitsConfig {
    tor_allowed: bool,
    ineligible_countries: Vec<String>,
    daily_limit: i64,
    ip_daily_limit: i64,
    ip_weekly_limit: i64,
}

pub struct RewardsClient {
    database: Database,
    stream_producer: StreamProducer,
    ip_security_client: IpSecurityClient,
}

impl RewardsClient {
    pub fn new(database: Database, stream_producer: StreamProducer, ip_security_client: IpSecurityClient) -> Self {
        Self {
            database,
            stream_producer,
            ip_security_client,
        }
    }

    pub fn get_rewards(&mut self, address: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        match self.database.client()?.rewards().get_reward_by_address(address) {
            Ok(rewards) => Ok(rewards),
            Err(storage::DatabaseError::NotFound) => Ok(Rewards::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_rewards_events(&mut self, address: &str) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_events_by_address(address)?)
    }

    pub fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_rewards_leaderboard()?)
    }

    pub fn get_rewards_redemption_option(&mut self, code: &str) -> Result<RewardRedemptionOption, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.rewards_redemptions().get_redemption_option(code)?)
    }

    pub async fn create_referral(&mut self, address: &str, code: &str, device_id: i32, ip_address: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let limit = self.database.client()?.config().get_config_i64(ConfigKey::UsernameCreationPerIp)?;
        self.ip_security_client.check_username_creation_limit(ip_address, limit).await?;

        let (rewards, event_id) = self.database.client()?.rewards().create_reward(address, code, device_id)?;
        self.ip_security_client.record_username_creation(ip_address).await?;
        self.publish_events(vec![event_id]).await?;
        Ok(rewards)
    }

    #[allow(dead_code)]
    pub fn change_username(&mut self, address: &str, new_username: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().change_username(address, new_username)?)
    }

    pub async fn use_referral_code(&mut self, auth: &VerifiedAuth, code: &str, ip_address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let referrer_username = self
            .database
            .client()?
            .rewards()
            .get_referrer_username(code)?
            .ok_or_else(|| ReferralValidationError::CodeDoesNotExist)?;

        match self.process_referral(auth, &referrer_username, ip_address).await {
            Ok(()) => Ok(()),
            Err(e) => Err(RewardsError::Referral(e.user_message()).into()),
        }
    }

    async fn process_referral(&mut self, auth: &VerifiedAuth, referrer_username: &str, ip_address: &str) -> Result<(), ReferralError> {
        self.check_referrer_limits(referrer_username)?;
        let invite_event = self.validate_referral_usage(auth, referrer_username)?;

        let ip_result = self.ip_security_client.check_ip(ip_address).await?;

        let mut client = self.database.client()?;
        let limits_config = Self::load_referral_limits_config(client.config())?;
        let risk_score_config = Self::load_risk_score_config(client.config())?;
        let lookback_days = client.config().get_config_i64(ConfigKey::ReferralRiskScoreLookbackDays)?;
        let since = now().days_ago(lookback_days);

        let referrer_verified = client.rewards().is_verified_by_username(referrer_username)?;

        let scoring_input = RiskScoringInput {
            username: referrer_username.to_string(),
            device_id: auth.device.id,
            device_platform: auth.device.platform.clone(),
            device_os: auth.device.os.clone().unwrap_or_default(),
            device_model: auth.device.model.clone().unwrap_or_default(),
            device_locale: auth.device.locale.clone(),
            ip_result,
            referrer_verified,
        };

        let signal_input = scoring_input.to_signal_input();
        let fingerprint = signal_input.generate_fingerprint();

        if !limits_config.tor_allowed && scoring_input.ip_result.is_tor {
            return Err(ReferralError::IpTorNotAllowed);
        }

        if limits_config.ineligible_countries.contains(&scoring_input.ip_result.country_code) {
            return Err(ReferralError::IpCountryIneligible(scoring_input.ip_result.country_code.clone()));
        }

        let daily_count = client.count_signals_since(None, now().days_ago(1))?;
        if daily_count >= limits_config.daily_limit {
            return Err(ReferralError::LimitReached(ConfigKey::ReferralUseDailyLimit));
        }

        let ip_daily_count = client.count_signals_since(Some(&scoring_input.ip_result.ip_address), now().days_ago(1))?;
        if ip_daily_count >= limits_config.ip_daily_limit {
            return Err(ReferralError::LimitReached(ConfigKey::ReferralPerIpDaily));
        }

        let ip_weekly_count = client.count_signals_since(Some(&scoring_input.ip_result.ip_address), now().days_ago(7))?;
        if ip_weekly_count >= limits_config.ip_weekly_limit {
            return Err(ReferralError::LimitReached(ConfigKey::ReferralPerIpWeekly));
        }

        let existing_signals = client.get_matching_risk_signals(
            &fingerprint,
            &signal_input.ip_address,
            &signal_input.ip_isp,
            &signal_input.device_model,
            signal_input.device_id,
            since,
        )?;

        let risk_result = evaluate_risk(&scoring_input, &existing_signals, &risk_score_config);
        let risk_signal_id = client.add_risk_signal(risk_result.signal)?;

        if !risk_result.score.is_allowed {
            let error = ReferralError::RiskScoreExceeded {
                score: risk_result.score.score,
                max_allowed: risk_score_config.max_allowed_score,
            };
            client
                .rewards()
                .add_referral_attempt(referrer_username, &auth.address, auth.device.id, Some(risk_signal_id), &error.to_string())?;
            return Err(error);
        }

        let event_ids = client
            .rewards()
            .create_referral_use(&auth.address, referrer_username, auth.device.id, risk_signal_id, invite_event)?;

        if let Err(e) = self.publish_events(event_ids).await {
            let error: ReferralError = e.into();
            client
                .rewards()
                .add_referral_attempt(referrer_username, &auth.address, auth.device.id, Some(risk_signal_id), &error.to_string())?;
            return Err(error);
        }

        Ok(())
    }

    fn check_referrer_limits(&mut self, referrer_code: &str) -> Result<(), ReferralError> {
        let current = now();
        let mut client = self.database.client()?;

        let daily_limit = client.config().get_config_i64(ConfigKey::ReferralPerUserDaily)?;
        if client.rewards().count_referrals_since(referrer_code, current.days_ago(1))? >= daily_limit {
            return Err(ReferralError::ReferrerLimitReached("daily".to_string()));
        }

        let weekly_limit = client.config().get_config_i64(ConfigKey::ReferralPerUserWeekly)?;
        if client.rewards().count_referrals_since(referrer_code, current.days_ago(7))? >= weekly_limit {
            return Err(ReferralError::ReferrerLimitReached("weekly".to_string()));
        }

        Ok(())
    }

    fn validate_referral_usage(&mut self, auth: &VerifiedAuth, referrer_username: &str) -> Result<RewardEventType, ReferralError> {
        let mut client = self.database.client()?;
        let first_subscription_date = client.rewards().get_first_subscription_date(vec![auth.address.clone()])?;

        let is_new_device = auth.device.created_at.is_within_days(REFERRAL_ELIGIBILITY_DAYS);
        let is_new_subscription = first_subscription_date.map(|d| d.is_within_days(REFERRAL_ELIGIBILITY_DAYS)).unwrap_or(true);

        client.rewards().validate_referral_use(&auth.address, referrer_username, auth.device.id)?;

        Ok(if is_new_device && is_new_subscription {
            RewardEventType::InviteNew
        } else {
            RewardEventType::InviteExisting
        })
    }

    fn load_referral_limits_config(config: &mut dyn storage::ConfigRepository) -> Result<ReferralLimitsConfig, storage::DatabaseError> {
        Ok(ReferralLimitsConfig {
            tor_allowed: config.get_config_bool(ConfigKey::ReferralIpTorAllowed)?,
            ineligible_countries: config.get_config_vec_string(ConfigKey::ReferralIneligibleCountries)?,
            daily_limit: config.get_config_i64(ConfigKey::ReferralUseDailyLimit)?,
            ip_daily_limit: config.get_config_i64(ConfigKey::ReferralPerIpDaily)?,
            ip_weekly_limit: config.get_config_i64(ConfigKey::ReferralPerIpWeekly)?,
        })
    }

    fn load_risk_score_config(config: &mut dyn storage::ConfigRepository) -> Result<RiskScoreConfig, storage::DatabaseError> {
        Ok(RiskScoreConfig {
            fingerprint_match_score: config.get_config_i64(ConfigKey::ReferralRiskScoreFingerprintMatch)?,
            ip_reuse_score: config.get_config_i64(ConfigKey::ReferralRiskScoreIpReuse)?,
            isp_model_match_score: config.get_config_i64(ConfigKey::ReferralRiskScoreIspModelMatch)?,
            device_id_reuse_score: config.get_config_i64(ConfigKey::ReferralRiskScoreDeviceIdReuse)?,
            ineligible_ip_type_score: config.get_config_i64(ConfigKey::ReferralRiskScoreIneligibleIpType)?,
            blocked_ip_types: config.get_config_vec_string(ConfigKey::ReferralBlockedIpTypes)?,
            blocked_ip_type_penalty: config.get_config_i64(ConfigKey::ReferralBlockedIpTypePenalty)?,
            max_abuse_score: config.get_config_i64(ConfigKey::ReferralMaxAbuseScore)?,
            penalty_isps: config.get_config_vec_string(ConfigKey::ReferralPenaltyIsps)?,
            isp_penalty_score: config.get_config_i64(ConfigKey::ReferralIspPenaltyScore)?,
            verified_user_reduction: config.get_config_i64(ConfigKey::ReferralRiskScoreVerifiedUserReduction)?,
            max_allowed_score: config.get_config_i64(ConfigKey::ReferralRiskScoreMaxAllowed)?,
            same_referrer_pattern_threshold: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternThreshold)?,
            same_referrer_pattern_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternPenalty)?,
            same_referrer_fingerprint_threshold: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintThreshold)?,
            same_referrer_fingerprint_penalty: config.get_config_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintPenalty)?,
        })
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer
            .publish_rewards_events(event_ids.into_iter().map(RewardsNotificationPayload::new).collect())
            .await?;
        Ok(())
    }
}

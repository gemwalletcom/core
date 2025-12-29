use std::error::Error;

use gem_rewards::{IpCheckConfig, IpSecurityClient, RewardsError, RiskScoreConfig, RiskScoringInput, evaluate_risk};
use primitives::rewards::RewardRedemptionOption;
use primitives::{ConfigKey, NaiveDateTimeExt, ReferralLeaderboard, RewardEvent, RewardEventType, Rewards, now};
use storage::{Database, RiskSignalsRepository};
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

use crate::auth::VerifiedAuth;

const REFERRAL_ELIGIBILITY_DAYS: i64 = 7;

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
        if !self.database.client()?.rewards().referral_code_exists(code)? {
            return Err(RewardsError::Referral("Referral code does not exist".to_string()).into());
        }

        match self.process_referral(auth, code, ip_address).await {
            Ok(()) => Ok(()),
            Err((risk_signal_id, e)) => {
                self.database
                    .client()?
                    .rewards()
                    .add_referral_attempt(code, &auth.address, auth.device.id, risk_signal_id, &e.to_string())?;
                Err(e)
            }
        }
    }

    async fn process_referral(&mut self, auth: &VerifiedAuth, code: &str, ip_address: &str) -> Result<(), (Option<i32>, Box<dyn Error + Send + Sync>)> {
        self.check_referrer_limits(code).map_err(|e| (None, e))?;
        let invite_event = self.validate_referral_usage(auth, code).map_err(|e| (None, e))?;

        let mut client = self.database.client().map_err(|e| (None, e.into()))?;
        let ip_check_config = IpCheckConfig {
            confidence_score_threshold: client
                .config()
                .get_config_i64(ConfigKey::ReferralIpConfidenceScoreThreshold)
                .map_err(|e| (None, e.into()))?,
            ineligible_usage_types: client
                .config()
                .get_config_vec_string(ConfigKey::ReferralIpIneligibleUsageTypes)
                .map_err(|e| (None, e.into()))?,
        };

        let ip_result = self.ip_security_client.check_ip(ip_address, &ip_check_config).await.map_err(|e| (None, e))?;

        let is_ip_eligible = !ip_result.is_suspicious(&ip_check_config);
        Self::check_ip_eligibility(&mut client, ip_address, is_ip_eligible, &ip_result.country_code).map_err(|e| (None, e))?;

        let risk_score_config = RiskScoreConfig {
            fingerprint_match_score: client
                .config()
                .get_config_i64(ConfigKey::ReferralRiskScoreFingerprintMatch)
                .map_err(|e| (None, e.into()))? as i32,
            ip_reuse_score: client
                .config()
                .get_config_i64(ConfigKey::ReferralRiskScoreIpReuse)
                .map_err(|e| (None, e.into()))? as i32,
            isp_model_match_score: client
                .config()
                .get_config_i64(ConfigKey::ReferralRiskScoreIspModelMatch)
                .map_err(|e| (None, e.into()))? as i32,
            device_id_reuse_score: client
                .config()
                .get_config_i64(ConfigKey::ReferralRiskScoreDeviceIdReuse)
                .map_err(|e| (None, e.into()))? as i32,
            max_allowed_score: client
                .config()
                .get_config_i64(ConfigKey::ReferralRiskScoreMaxAllowed)
                .map_err(|e| (None, e.into()))? as i32,
        };

        let lookback_days = client
            .config()
            .get_config_i64(ConfigKey::ReferralRiskScoreLookbackDays)
            .map_err(|e| (None, e.into()))?;
        let since = now().days_ago(lookback_days);

        let scoring_input = RiskScoringInput {
            username: auth.address.clone(),
            device_id: auth.device.id,
            device_platform: auth.device.platform.clone(),
            device_os: auth.device.os.clone().unwrap_or_default(),
            device_model: auth.device.model.clone().unwrap_or_default(),
            device_locale: auth.device.locale.clone(),
            ip_result,
        };

        let signal_input = scoring_input.to_signal_input();
        let fingerprint = signal_input.generate_fingerprint();
        let existing_signals = client
            .get_matching_risk_signals(
                &fingerprint,
                &signal_input.ip_address,
                &signal_input.ip_isp,
                &signal_input.device_model,
                signal_input.device_id,
                since,
            )
            .map_err(|e| (None, e.into()))?;

        let risk_result = evaluate_risk(&scoring_input, &existing_signals, &risk_score_config);
        let risk_signal_id = client.add_risk_signal(risk_result.signal).map_err(|e| (None, e.into()))?;

        if !risk_result.score_result.is_allowed {
            return Err((
                Some(risk_signal_id),
                RewardsError::Referral("Unable to verify referral eligibility".to_string()).into(),
            ));
        }

        let event_ids = client
            .rewards()
            .create_referral_use(&auth.address, code, auth.device.id, risk_signal_id, invite_event)
            .map_err(|e| (Some(risk_signal_id), e.into()))?;

        self.publish_events(event_ids).await.map_err(|e| (Some(risk_signal_id), e))?;

        Ok(())
    }

    fn check_referrer_limits(&mut self, referrer_code: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;
        let current = now();

        let daily_limit = client.config().get_config_i64(ConfigKey::ReferralPerUserDaily)?;
        if client.rewards().count_referrals_since(referrer_code, current.days_ago(1))? >= daily_limit {
            return Err(RewardsError::Referral("Referrer daily limit reached".to_string()).into());
        }

        let weekly_limit = client.config().get_config_i64(ConfigKey::ReferralPerUserWeekly)?;
        if client.rewards().count_referrals_since(referrer_code, current.days_ago(7))? >= weekly_limit {
            return Err(RewardsError::Referral("Referrer weekly limit reached".to_string()).into());
        }

        Ok(())
    }

    fn validate_referral_usage(&mut self, auth: &VerifiedAuth, code: &str) -> Result<RewardEventType, Box<dyn Error + Send + Sync>> {
        let first_subscription_date = self.database.client()?.rewards().get_first_subscription_date(vec![auth.address.clone()])?;

        let is_new_device = auth.device.created_at.is_within_days(REFERRAL_ELIGIBILITY_DAYS);
        let is_new_subscription = first_subscription_date.map(|d| d.is_within_days(REFERRAL_ELIGIBILITY_DAYS)).unwrap_or(true);

        self.database.client()?.rewards().validate_referral_use(&auth.address, code, auth.device.id)?;

        Ok(if is_new_device && is_new_subscription {
            RewardEventType::InviteNew
        } else {
            RewardEventType::InviteExisting
        })
    }

    fn check_ip_eligibility(
        client: &mut storage::DatabaseClient,
        ip_address: &str,
        is_ip_eligible: bool,
        country: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !is_ip_eligible {
            return Err(RewardsError::Referral(format!("IP not eligible, country: {}", country)).into());
        }

        if client
            .config()
            .get_config_vec_string(ConfigKey::ReferralIneligibleCountries)?
            .contains(&country.to_string())
        {
            return Err(RewardsError::Referral(format!("Country not eligible: {}", country)).into());
        }

        let current = now();

        let global_daily_limit = client.config().get_config_i64(ConfigKey::ReferralUseDailyLimit)?;
        if client.count_signals_since(None, current.days_ago(1))? >= global_daily_limit {
            return Err(RewardsError::Referral("Global daily limit exceeded".to_string()).into());
        }

        let daily_limit = client.config().get_config_i64(ConfigKey::ReferralPerIpDaily)?;
        if client.count_signals_since(Some(ip_address), current.days_ago(1))? >= daily_limit {
            return Err(RewardsError::Referral("Daily limit exceeded".to_string()).into());
        }

        let weekly_limit = client.config().get_config_i64(ConfigKey::ReferralPerIpWeekly)?;
        if client.count_signals_since(Some(ip_address), current.days_ago(7))? >= weekly_limit {
            return Err(RewardsError::Referral("Weekly limit exceeded".to_string()).into());
        }

        Ok(())
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer
            .publish_rewards_events(event_ids.into_iter().map(RewardsNotificationPayload::new).collect())
            .await?;
        Ok(())
    }
}

use std::error::Error;

use gem_rewards::{IpCheckConfig, IpSecurityClient, RewardsError};
use primitives::rewards::RewardRedemptionOption;
use primitives::{ConfigKey, NaiveDateTimeExt, ReferralLeaderboard, RewardEvent, RewardEventType, Rewards, now};
use storage::Database;
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

    pub async fn create_referral(&mut self, address: &str, code: &str, ip_address: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let limit = self.database.client()?.config().get_config_i64(ConfigKey::UsernameCreationPerIp)?;
        self.ip_security_client.check_username_creation_limit(ip_address, limit).await?;

        let (rewards, event_id) = self.database.client()?.rewards().create_reward(address, code)?;
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
            Err(e) => {
                self.database
                    .client()?
                    .rewards()
                    .add_referral_attempt(code, &auth.address, auth.device.id, ip_address, &e.to_string())?;
                Err(e)
            }
        }
    }

    async fn process_referral(&mut self, auth: &VerifiedAuth, code: &str, ip_address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.check_referrer_limits(code)?;
        let invite_event = self.validate_referral_usage(auth, code)?;

        let mut client = self.database.client()?;
        let ip_check_config = IpCheckConfig {
            confidence_score_threshold: client.config().get_config_i64(ConfigKey::ReferralIpConfidenceScoreThreshold)?,
            ineligible_usage_types: client.config().get_config_vec_string(ConfigKey::ReferralIpIneligibleUsageTypes)?,
        };

        let (is_ip_eligible, country) = self.ip_security_client.check_eligibility(ip_address, &ip_check_config).await?;

        self.check_ip_eligibility(ip_address, is_ip_eligible, &country).await?;
        self.database
            .client()?
            .rewards()
            .create_referral_use(&auth.address, code, auth.device.id, ip_address, invite_event)?;
        self.ip_security_client.record_referral_usage(ip_address).await?;

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

    async fn check_ip_eligibility(&mut self, ip_address: &str, is_ip_eligible: bool, country: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !is_ip_eligible {
            return Err(RewardsError::Referral(format!("IP not eligible, country: {}", country)).into());
        }

        let mut client = self.database.client()?;

        if client
            .config()
            .get_config_vec_string(ConfigKey::ReferralIneligibleCountries)?
            .contains(&country.to_string())
        {
            return Err(RewardsError::Referral(format!("Country not eligible: {}", country)).into());
        }

        let daily_limit = client.config().get_config_i64(ConfigKey::ReferralPerIpDaily)?;
        let weekly_limit = client.config().get_config_i64(ConfigKey::ReferralPerIpWeekly)?;
        let global_daily_limit = client.config().get_config_i64(ConfigKey::ReferralUseDailyLimit)?;
        self.ip_security_client
            .check_rate_limits(ip_address, daily_limit, weekly_limit, global_daily_limit)
            .await?;

        Ok(())
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer
            .publish_rewards_events(event_ids.into_iter().map(RewardsNotificationPayload::new).collect())
            .await?;
        Ok(())
    }
}

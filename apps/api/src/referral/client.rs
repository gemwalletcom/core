use gem_rewards::{IpSecurityClient, RewardsError};
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

    pub fn get_rewards(&mut self, address: &str) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        match self.database.client()?.rewards().get_reward_by_address(address) {
            Ok(rewards) => Ok(rewards),
            Err(storage::DatabaseError::NotFound) => Ok(Rewards::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_rewards_events(&mut self, address: &str) -> Result<Vec<RewardEvent>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_events_by_address(address)?)
    }

    pub fn get_rewards_leaderboard(&mut self) -> Result<ReferralLeaderboard, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_rewards_leaderboard()?)
    }

    pub async fn create_referral(&mut self, address: &str, code: &str, ip_address: &str) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        let limit = self.database.client()?.config().get_config_i64(ConfigKey::UsernameCreationPerIp)?;
        self.ip_security_client.check_username_creation_limit(ip_address, limit).await?;

        let (rewards, event_id) = self.database.client()?.rewards().create_reward(address, code)?;
        self.ip_security_client.record_username_creation(ip_address).await?;
        self.publish_events(vec![event_id]).await?;
        Ok(rewards)
    }

    #[allow(dead_code)]
    pub fn change_username(&mut self, address: &str, new_username: &str) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().change_username(address, new_username)?)
    }

    pub async fn use_referral_code(&mut self, auth: &VerifiedAuth, code: &str, ip_address: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.database.client()?.rewards().referral_code_exists(code)? {
            return Err(RewardsError::Referral("Referral code does not exist".to_string()).into());
        }

        let first_subscription_date = self.database.client()?.rewards().get_first_subscription_date(vec![auth.address.clone()])?;

        let is_new_device = auth.device.created_at.is_within_days(REFERRAL_ELIGIBILITY_DAYS);
        let is_new_subscription = first_subscription_date
            .map(|date| date.is_within_days(REFERRAL_ELIGIBILITY_DAYS))
            .unwrap_or(true);

        let is_new_user = is_new_device && is_new_subscription;

        let invite_event = if is_new_user {
            RewardEventType::InviteNew
        } else {
            RewardEventType::InviteExisting
        };

        let (is_ip_eligible, country) = self.ip_security_client.check_eligibility(ip_address).await?;

        if let Err(err) = self.check_referral_eligibility(code, ip_address, is_ip_eligible, &country).await {
            self.add_referral_attempt(code, &auth.address, &country, auth.device.id, ip_address, &err.to_string())?;
            return Err(err);
        }

        let event_ids = match self
            .database
            .client()?
            .rewards()
            .use_referral_code(&auth.address, code, auth.device.id, ip_address, invite_event)
        {
            Ok(ids) => ids,
            Err(err) => {
                self.add_referral_attempt(code, &auth.address, &country, auth.device.id, ip_address, &err.to_string())?;
                return Err(err.into());
            }
        };

        self.ip_security_client.record_referral_usage(ip_address).await?;
        self.publish_events(event_ids).await?;
        Ok(())
    }

    async fn check_referral_eligibility(
        &mut self,
        referrer_code: &str,
        ip_address: &str,
        is_ip_eligible: bool,
        country: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !is_ip_eligible {
            return Err(RewardsError::Referral("IP not eligible".to_string()).into());
        }

        let mut client = self.database.client()?;

        let ineligible_countries = client.config().get_config_vec_string(ConfigKey::ReferralIneligibleCountries)?;
        if ineligible_countries.contains(&country.to_string()) {
            return Err(RewardsError::Referral(format!("Country {} not eligible", country)).into());
        }

        let current = now();

        let per_user_daily_limit = client.config().get_config_i64(ConfigKey::ReferralPerUserDaily)?;
        let per_user_daily_count = client.rewards().count_referrals_since(referrer_code, current.days_ago(1))?;
        if per_user_daily_count >= per_user_daily_limit {
            return Err(RewardsError::Referral("Referrer daily limit reached".to_string()).into());
        }

        let per_user_weekly_limit = client.config().get_config_i64(ConfigKey::ReferralPerUserWeekly)?;
        let per_user_weekly_count = client.rewards().count_referrals_since(referrer_code, current.days_ago(7))?;
        if per_user_weekly_count >= per_user_weekly_limit {
            return Err(RewardsError::Referral("Referrer weekly limit reached".to_string()).into());
        }

        let daily_limit = client.config().get_config_i64(ConfigKey::ReferralPerIpDaily)?;
        let weekly_limit = client.config().get_config_i64(ConfigKey::ReferralPerIpWeekly)?;
        let global_daily_limit = client.config().get_config_i64(ConfigKey::ReferralUseDailyLimit)?;
        self.ip_security_client
            .check_rate_limits(ip_address, daily_limit, weekly_limit, global_daily_limit)
            .await?;

        Ok(())
    }

    fn add_referral_attempt(
        &mut self,
        referrer_username: &str,
        referred_address: &str,
        country_code: &str,
        device_id: i32,
        ip_address: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.database
            .client()?
            .rewards()
            .add_referral_attempt(referrer_username, referred_address, country_code, device_id, ip_address, reason)?;
        Ok(())
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payloads: Vec<_> = event_ids.into_iter().map(RewardsNotificationPayload::new).collect();
        self.stream_producer.publish_rewards_events(payloads).await?;
        Ok(())
    }
}

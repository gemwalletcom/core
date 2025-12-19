use std::error::Error;

use cacher::{CacheKey, CacherClient};

use crate::abuseipdb_client::{AbuseIPDBClient, AbuseIPDBData};

#[derive(Clone)]
pub struct IpSecurityClient {
    abuseipdb_client: AbuseIPDBClient,
    cacher: CacherClient,
}

impl IpSecurityClient {
    pub fn new(abuseipdb_client: AbuseIPDBClient, cacher: CacherClient) -> Self {
        Self { abuseipdb_client, cacher }
    }

    pub async fn check_ip(&self, ip_address: &str) -> Result<AbuseIPDBData, Box<dyn Error + Send + Sync>> {
        self.cacher
            .get_or_set_cached(CacheKey::ReferralIpCheck(ip_address), || async {
                self.abuseipdb_client.check_ip(ip_address).await
            })
            .await
    }

    pub async fn check_eligibility(&self, ip_address: &str) -> Result<(bool, String), Box<dyn Error + Send + Sync>> {
        let ip_data = self.check_ip(ip_address).await?;
        Ok((!ip_data.is_suspicious(), ip_data.country_code))
    }

    pub async fn check_rate_limits(&self, ip_address: &str, daily_limit: i64, weekly_limit: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.cacher.get_cached_counter(CacheKey::ReferralDailyLimit(ip_address)).await? >= daily_limit {
            return Err(crate::RewardsError::Referral("Daily limit exceeded".to_string()).into());
        }

        if self.cacher.get_cached_counter(CacheKey::ReferralWeeklyLimit(ip_address)).await? >= weekly_limit {
            return Err(crate::RewardsError::Referral("Weekly limit exceeded".to_string()).into());
        }

        Ok(())
    }

    pub async fn record_referral_usage(&self, ip_address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.increment_cached(CacheKey::ReferralDailyLimit(ip_address)).await?;
        self.cacher.increment_cached(CacheKey::ReferralWeeklyLimit(ip_address)).await?;
        Ok(())
    }
}

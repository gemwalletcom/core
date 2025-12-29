use std::error::Error;

use cacher::{CacheKey, CacherClient};

use crate::abuseipdb::AbuseIPDBClient;
use crate::model::{IpCheckConfig, IpCheckResult};

#[derive(Clone)]
pub struct IpSecurityClient {
    abuseipdb_client: AbuseIPDBClient,
    cacher: CacherClient,
}

impl IpSecurityClient {
    pub fn new(abuseipdb_client: AbuseIPDBClient, cacher: CacherClient) -> Self {
        Self { abuseipdb_client, cacher }
    }

    pub async fn check_ip(&self, ip_address: &str, config: &IpCheckConfig) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        let ip_data = self
            .cacher
            .get_or_set_cached(CacheKey::ReferralIpCheck(ip_address), || async {
                self.abuseipdb_client.check_ip(ip_address).await
            })
            .await?;
        Ok(ip_data.as_ip_check_result(&config.ineligible_usage_types))
    }

    pub async fn check_eligibility(&self, ip_address: &str, config: &IpCheckConfig) -> Result<(bool, String), Box<dyn Error + Send + Sync>> {
        let ip_result = self.check_ip(ip_address, config).await?;
        Ok((!ip_result.is_suspicious(config), ip_result.country_code))
    }

    pub async fn check_username_creation_limit(&self, ip_address: &str, limit: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.cacher.get_cached_counter(CacheKey::UsernameCreationPerIp(ip_address)).await? >= limit {
            return Err(crate::RewardsError::Username("Too many usernames created from this IP".to_string()).into());
        }
        Ok(())
    }

    pub async fn record_username_creation(&self, ip_address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.increment_cached(CacheKey::UsernameCreationPerIp(ip_address)).await?;
        Ok(())
    }
}

use std::error::Error;

use cacher::{CacheKey, CacherClient};

use crate::abuseipdb::AbuseIPDBClient;
use crate::model::IpCheckResult;

#[derive(Clone)]
pub struct IpSecurityClient {
    abuseipdb_client: AbuseIPDBClient,
    cacher: CacherClient,
}

impl IpSecurityClient {
    pub fn new(abuseipdb_client: AbuseIPDBClient, cacher: CacherClient) -> Self {
        Self { abuseipdb_client, cacher }
    }

    pub async fn check_ip(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        let ip_data = self
            .cacher
            .get_or_set_cached(CacheKey::ReferralIpCheck(ip_address), || async {
                self.abuseipdb_client.check_ip(ip_address).await
            })
            .await?;
        Ok(ip_data.as_ip_check_result())
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

    pub async fn check_username_creation_device_limit(&self, device_id: i32, limit: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.cacher.get_cached_counter(CacheKey::UsernameCreationPerDevice(device_id)).await? >= limit {
            return Err(crate::RewardsError::Username("Too many usernames created from this device".to_string()).into());
        }
        Ok(())
    }

    pub async fn record_username_creation_device(&self, device_id: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.increment_cached(CacheKey::UsernameCreationPerDevice(device_id)).await?;
        Ok(())
    }
}

use std::error::Error;

use cacher::{CacheKey, CacherClient};
use primitives::ConfigKey;

use crate::abuseipdb::AbuseIPDBClient;
use crate::model::IpCheckResult;
use crate::UsernameError;

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

    pub async fn check_username_creation_limits(
        &self,
        ip_address: &str,
        device_id: i32,
        global_daily_limit: i64,
        ip_limit: i64,
        device_limit: i64,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let global_count = self.cacher.get_cached_counter(CacheKey::UsernameCreationGlobalDaily).await?;
        if global_count >= global_daily_limit {
            return Err(UsernameError::LimitReached(ConfigKey::UsernameCreationGlobalDailyLimit).into());
        }

        let ip_count = self.cacher.get_cached_counter(CacheKey::UsernameCreationPerIp(ip_address)).await?;
        if ip_count >= ip_limit {
            return Err(UsernameError::LimitReached(ConfigKey::UsernameCreationPerIp).into());
        }

        let device_count = self.cacher.get_cached_counter(CacheKey::UsernameCreationPerDevice(device_id)).await?;
        if device_count >= device_limit {
            return Err(UsernameError::LimitReached(ConfigKey::UsernameCreationPerDevice).into());
        }

        Ok(())
    }

    pub async fn check_username_creation_country_limit(&self, country_code: &str, country_daily_limit: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let country_count = self.cacher.get_cached_counter(CacheKey::UsernameCreationPerCountryDaily(country_code)).await?;
        if country_count >= country_daily_limit {
            return Err(UsernameError::LimitReached(ConfigKey::UsernameCreationPerCountryDailyLimit).into());
        }

        Ok(())
    }

    pub fn check_username_creation_country_eligibility(&self, country_code: &str, ineligible_countries: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if ineligible_countries.contains(&country_code.to_string()) {
            return Err(UsernameError::LimitReached(ConfigKey::ReferralIneligibleCountries).into());
        }
        Ok(())
    }

    pub fn check_username_creation_ip_type(&self, usage_type: &str, blocked_ip_types: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let is_blocked = blocked_ip_types.iter().any(|t| usage_type.contains(t));
        if is_blocked {
            return Err(UsernameError::LimitReached(ConfigKey::ReferralBlockedIpTypes).into());
        }
        Ok(())
    }

    pub async fn record_username_creation(&self, country_code: &str, ip_address: &str, device_id: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.increment_cached(CacheKey::UsernameCreationGlobalDaily).await?;
        self.cacher.increment_cached(CacheKey::UsernameCreationPerCountryDaily(country_code)).await?;
        self.cacher.increment_cached(CacheKey::UsernameCreationPerIp(ip_address)).await?;
        self.cacher.increment_cached(CacheKey::UsernameCreationPerDevice(device_id)).await?;
        Ok(())
    }
}

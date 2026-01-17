use std::error::Error;
use std::sync::Arc;

use cacher::{CacheKey, CacherClient};
use primitives::{ConfigKey, IpUsageType};

use crate::ip_check_provider::IpCheckProvider;
use crate::model::IpCheckResult;
use crate::UsernameError;

#[derive(Clone)]
pub struct IpSecurityClient {
    providers: Vec<Arc<dyn IpCheckProvider>>,
    cacher: CacherClient,
}

impl IpSecurityClient {
    pub fn new(providers: Vec<Arc<dyn IpCheckProvider>>, cacher: CacherClient) -> Self {
        Self { providers, cacher }
    }

    pub async fn check_ip(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        self.cacher
            .get_or_set_cached(CacheKey::ReferralIpCheck(ip_address), || async { self.check_ip_with_fallback(ip_address).await })
            .await
    }

    async fn check_ip_with_fallback(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        let mut last_error: Option<Box<dyn Error + Send + Sync>> = None;

        for provider in &self.providers {
            match provider.check_ip(ip_address).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "No IP check providers configured".into()))
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

    pub fn check_username_creation_ip_type(&self, usage_type: IpUsageType, blocked_ip_types: &[IpUsageType]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if blocked_ip_types.contains(&usage_type) {
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

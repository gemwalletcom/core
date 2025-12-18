use std::error::Error;

use alloy_primitives::hex;
use cacher::CacherClient;
use sha2::{Digest, Sha256};

use crate::abuseipdb_client::{AbuseIPDBClient, AbuseIPDBData};

const IP_CHECK_CACHE_TTL_SECONDS: u64 = 30 * 24 * 60 * 60; // 30 days
const SECONDS_PER_DAY: i64 = 24 * 60 * 60;
const SECONDS_PER_WEEK: i64 = 7 * SECONDS_PER_DAY;

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
        let ip_hash = hash_ip(ip_address);

        self.cacher
            .get_or_set_value(
                &ip_check_key(&ip_hash),
                || async { self.abuseipdb_client.check_ip(ip_address).await },
                Some(IP_CHECK_CACHE_TTL_SECONDS),
            )
            .await
    }

    pub async fn check_eligibility(&self, ip_address: &str) -> Result<(bool, String), Box<dyn Error + Send + Sync>> {
        let ip_data = self.check_ip(ip_address).await?;
        Ok((!ip_data.is_suspicious(), ip_data.country_code))
    }

    pub async fn check_rate_limits(&self, ip_address: &str, daily_limit: i64, weekly_limit: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ip_hash = hash_ip(ip_address);

        if self.cacher.get_counter(&daily_key(&ip_hash)).await? >= daily_limit {
            return Err(crate::RewardsError::Referral("Daily limit exceeded".to_string()).into());
        }

        if self.cacher.get_counter(&weekly_key(&ip_hash)).await? >= weekly_limit {
            return Err(crate::RewardsError::Referral("Weekly limit exceeded".to_string()).into());
        }

        Ok(())
    }

    pub async fn record_referral_usage(&self, ip_address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ip_hash = hash_ip(ip_address);
        self.cacher.increment_with_ttl(&daily_key(&ip_hash), SECONDS_PER_DAY).await?;
        self.cacher.increment_with_ttl(&weekly_key(&ip_hash), SECONDS_PER_WEEK).await?;
        Ok(())
    }
}

fn hash_ip(ip_address: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ip_address.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..])
}

fn ip_check_key(ip_hash: &str) -> String {
    format!("referral:ip_check:{}", ip_hash)
}

fn daily_key(ip_hash: &str) -> String {
    format!("referral:ip_daily:{}", ip_hash)
}

fn weekly_key(ip_hash: &str) -> String {
    format!("referral:ip_weekly:{}", ip_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_ip() {
        let hash1 = hash_ip("192.168.1.1");
        let hash2 = hash_ip("192.168.1.1");
        let hash3 = hash_ip("192.168.1.2");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 hex = 64 chars
    }
}

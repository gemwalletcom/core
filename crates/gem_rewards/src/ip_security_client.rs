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
        let cache_key = format!("ip_security:check:{}", ip_hash);

        self.cacher
            .get_or_set_value(
                &cache_key,
                || async { self.abuseipdb_client.check_ip(ip_address).await },
                Some(IP_CHECK_CACHE_TTL_SECONDS),
            )
            .await
    }

    pub async fn is_eligible(&self, ip_address: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let ip_data = self.check_ip(ip_address).await?;
        Ok(!ip_data.is_suspicious())
    }

    pub async fn can_use_referral(&self, ip_address: &str, daily_limit: i64, weekly_limit: i64) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let ip_hash = hash_ip(ip_address);

        let daily_key = format!("referral:ip_daily:{}", ip_hash);
        let daily_count = self.cacher.get_counter(&daily_key).await?;
        if daily_count >= daily_limit {
            return Ok(false);
        }

        let weekly_key = format!("referral:ip_weekly:{}", ip_hash);
        let weekly_count = self.cacher.get_counter(&weekly_key).await?;
        if weekly_count >= weekly_limit {
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn record_referral_usage(&self, ip_address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ip_hash = hash_ip(ip_address);

        let daily_key = format!("referral:ip_daily:{}", ip_hash);
        let weekly_key = format!("referral:ip_weekly:{}", ip_hash);

        self.cacher.increment_with_ttl(&daily_key, SECONDS_PER_DAY).await?;
        self.cacher.increment_with_ttl(&weekly_key, SECONDS_PER_WEEK).await?;

        Ok(())
    }
}

pub fn hash_ip(ip_address: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ip_address.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..])
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

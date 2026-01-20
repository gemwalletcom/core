use std::error::Error;

use async_trait::async_trait;

use super::model::AbuseIPDBResponse;
use crate::ip_check_provider::IpCheckProvider;
use crate::model::IpCheckResult;

#[derive(Clone)]
pub struct AbuseIPDBClient {
    client: reqwest::Client,
    url: String,
    api_key: String,
}

impl AbuseIPDBClient {
    pub fn new(url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            api_key,
        }
    }
}

#[async_trait]
impl IpCheckProvider for AbuseIPDBClient {
    fn name(&self) -> &'static str {
        "abuseipdb"
    }

    async fn check_ip(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/check", self.url);
        let response = self
            .client
            .get(&url)
            .header("Key", &self.api_key)
            .header("Accept", "application/json")
            .query(&[("ipAddress", ip_address)])
            .send()
            .await?
            .json::<AbuseIPDBResponse>()
            .await?;

        Ok(response.data.as_ip_check_result())
    }
}

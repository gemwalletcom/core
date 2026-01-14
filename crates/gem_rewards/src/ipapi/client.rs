use std::error::Error;

use async_trait::async_trait;

use super::model::IpApiResponse;
use crate::ip_check_provider::IpCheckProvider;
use crate::model::IpCheckResult;

#[derive(Clone)]
pub struct IpApiClient {
    client: reqwest::Client,
    url: String,
    api_key: String,
}

impl IpApiClient {
    pub fn new(url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            api_key,
        }
    }
}

#[async_trait]
impl IpCheckProvider for IpApiClient {
    fn name(&self) -> &'static str {
        "ipapi"
    }

    async fn check_ip(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/", self.url);
        let response = self
            .client
            .get(&url)
            .query(&[("q", ip_address), ("key", &self.api_key)])
            .send()
            .await?
            .json::<IpApiResponse>()
            .await?;

        Ok(response.as_ip_check_result())
    }
}

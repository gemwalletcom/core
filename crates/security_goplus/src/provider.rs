use crate::models::{Response, SecurityAddress};
use async_trait::async_trait;
use security_provider::{ScanProvider, ScanResult, ScanTarget};
use std::result::Result;

static PROVIDER_NAME: &str = "GoPlus";

pub struct GoPlusProvider {
    client: reqwest::Client,
    url: String,
}

impl GoPlusProvider {
    pub fn new(client: reqwest::Client, url: &str, _api_key: &str) -> Self {
        GoPlusProvider { client, url: url.into() }
    }
}

#[async_trait]
impl ScanProvider for GoPlusProvider {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn scan(&self, target: &ScanTarget) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        match target {
            ScanTarget::Address(target) => self.scan_address(target.chain.network_id(), target.address.clone()).await,
            ScanTarget::URL(_) => Ok(ScanResult {
                is_malicious: false,
                reason: None,
                provider: self.name().into(),
            }),
        }
    }
}

impl GoPlusProvider {
    async fn scan_address(&self, chain: &str, address: String) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let url: String = format!("{}/api/v1/address_security/{}", self.url, address);
        let query = [("chain_id", chain)];
        let response = self.client.get(&url).query(&query).send().await?.json::<Response<SecurityAddress>>().await?;

        Ok(ScanResult {
            is_malicious: response.result.is_malicious(),
            reason: None,
            provider: self.name().into(),
        })
    }
}

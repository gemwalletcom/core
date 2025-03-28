use crate::models::{Response, SecurityAddress};
use async_trait::async_trait;
use security_provider::{AddressTarget, ScanProvider, ScanResult};
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

    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let url: String = format!("{}/api/v1/address_security/{}", self.url, target.address);
        let query = [("chain_id", target.chain)];
        let response = self.client.get(&url).query(&query).send().await?.json::<Response<SecurityAddress>>().await?;

        Ok(ScanResult {
            target: target.clone(),
            is_malicious: response.result.is_malicious(),
            reason: None,
            provider: self.name().into(),
        })
    }

    async fn scan_url(&self, _target: &str) -> Result<ScanResult<String>, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }
}

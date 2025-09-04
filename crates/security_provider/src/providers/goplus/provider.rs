use crate::providers::goplus::models::{Response, SecurityAddress, SecurityToken};
use crate::{mapper, AddressTarget, ScanProvider, ScanResult, TokenTarget};
use async_trait::async_trait;
use gem_client::{build_path_with_query, Client, ReqwestClient};
use std::result::Result;

static PROVIDER_NAME: &str = "GoPlus";

pub struct GoPlusProvider {
    client: ReqwestClient,
}

impl GoPlusProvider {
    pub fn new(client: ReqwestClient, _api_key: &str) -> Self {
        GoPlusProvider { client }
    }
}

#[async_trait]
impl ScanProvider for GoPlusProvider {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let path: String = format!("/api/v1/address_security/{}", target.address);
        let query = vec![("chain_id", mapper::chain_to_provider_id(target.chain))];
        let url = build_path_with_query(&path, &query)?;
        let response = self.client.get::<Response<SecurityAddress>>(&url).await?;

        Ok(ScanResult {
            target: target.clone(),
            is_malicious: response.result.is_malicious(),
            reason: None,
            provider: self.name().into(),
        })
    }

    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let path: String = format!("/api/v1/token_security/{}", target.token_id);
        let query = vec![("chain_id", mapper::chain_to_provider_id(target.chain))];
        let url = build_path_with_query(&path, &query)?;
        let response = self.client.get::<Response<SecurityToken>>(&url).await?;

        Ok(ScanResult {
            target: target.clone(),
            is_malicious: response.result.is_malicious(),
            reason: if response.result.is_malicious() {
                Some("Token security risk detected".to_string())
            } else {
                None
            },
            provider: self.name().into(),
        })
    }

    async fn scan_url(&self, _target: &str) -> Result<ScanResult<String>, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }
}

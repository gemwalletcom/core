use crate::providers::goplus::models::{Response, SecurityAddress, SecurityToken};
use crate::{AddressTarget, ScanProvider, ScanResult, TokenTarget, mapper};
use async_trait::async_trait;
use gem_client::{Client, ReqwestClient, build_path_with_query};
use std::collections::HashMap;
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
        let path = format!("/api/v1/address_security/{}", target.address);
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
        let path = format!("/api/v1/token_security/{}", mapper::chain_to_provider_id(target.chain));
        let query = vec![("contract_addresses", target.token_id.as_str())];
        let url = build_path_with_query(&path, &query)?;
        let response = self.client.get::<Response<HashMap<String, SecurityToken>>>(&url).await?;

        let security_token = {
            let key = target.token_id.to_lowercase();
            response.result.get(&key).cloned().or_else(|| response.result.values().next().cloned())
        };

        let (is_malicious, reason) = match security_token {
            Some(tok) => {
                let mal = tok.is_malicious();
                let reason = if mal { Some("Token security risk detected".to_string()) } else { None };
                (mal, reason)
            }
            None => (false, Some("No token data found".to_string())),
        };

        Ok(ScanResult {
            target: target.clone(),
            is_malicious,
            reason,
            provider: self.name().into(),
        })
    }

    async fn scan_url(&self, _target: &str) -> Result<ScanResult<String>, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!()
    }
}

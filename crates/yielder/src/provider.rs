use std::{fmt, str::FromStr, sync::Arc};

use alloy_primitives::Address;
use async_trait::async_trait;
use primitives::{AssetId, Chain};

use crate::yo::YieldError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YieldProvider {
    Yo,
}

impl YieldProvider {
    pub fn name(&self) -> &'static str {
        match self {
            YieldProvider::Yo => "yo",
        }
    }
}

impl fmt::Display for YieldProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for YieldProvider {
    type Err = YieldError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "yo" => Ok(YieldProvider::Yo),
            other => Err(YieldError::new(format!("unknown yield provider {other}"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Yield {
    pub name: String,
    pub asset_id: AssetId,
    pub provider: YieldProvider,
    pub apy: Option<f64>,
}

impl Yield {
    pub fn new(name: impl Into<String>, asset_id: AssetId, provider: YieldProvider, apy: Option<f64>) -> Self {
        Self {
            name: name.into(),
            asset_id,
            provider,
            apy,
        }
    }
}

#[derive(Debug, Clone)]
pub struct YieldTransaction {
    pub chain: Chain,
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct YieldDetailsRequest {
    pub asset_id: AssetId,
    pub wallet_address: String,
}

#[derive(Debug, Clone)]
pub struct YieldPosition {
    pub asset_id: AssetId,
    pub provider: YieldProvider,
    pub vault_token_address: String,
    pub asset_token_address: String,
    pub vault_balance_value: Option<String>,
    pub asset_balance_value: Option<String>,
    pub apy: Option<f64>,
    pub rewards: Option<String>,
}

impl YieldPosition {
    pub fn new(asset_id: AssetId, provider: YieldProvider, share_token: Address, asset_token: Address) -> Self {
        Self {
            asset_id,
            provider,
            vault_token_address: share_token.to_string(),
            asset_token_address: asset_token.to_string(),
            vault_balance_value: None,
            asset_balance_value: None,
            apy: None,
            rewards: None,
        }
    }
}

#[async_trait]
pub trait YieldProviderClient: Send + Sync {
    fn provider(&self) -> YieldProvider;
    fn yields(&self, asset_id: &AssetId) -> Vec<Yield>;
    async fn deposit(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError>;
    async fn withdraw(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError>;
    async fn positions(&self, request: &YieldDetailsRequest) -> Result<YieldPosition, YieldError>;
    async fn yields_with_apy(&self, asset_id: &AssetId) -> Result<Vec<Yield>, YieldError> {
        Ok(self.yields(asset_id))
    }
}

#[derive(Default)]
pub struct Yielder {
    providers: Vec<Arc<dyn YieldProviderClient>>,
}

impl Yielder {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    pub fn with_providers(providers: Vec<Arc<dyn YieldProviderClient>>) -> Self {
        Self { providers }
    }

    pub fn add_provider<P>(&mut self, provider: P)
    where
        P: YieldProviderClient + 'static,
    {
        self.providers.push(Arc::new(provider));
    }

    pub fn add_provider_arc(&mut self, provider: Arc<dyn YieldProviderClient>) {
        self.providers.push(provider);
    }

    pub fn yields_for_asset(&self, asset_id: &AssetId) -> Vec<Yield> {
        self.providers.iter().flat_map(|provider| provider.yields(asset_id)).collect()
    }

    pub async fn yields_for_asset_with_apy(&self, asset_id: &AssetId) -> Result<Vec<Yield>, YieldError> {
        let mut yields = Vec::new();
        for provider in &self.providers {
            let mut provider_yields = provider.yields_with_apy(asset_id).await?;
            yields.append(&mut provider_yields);
        }
        Ok(yields)
    }

    pub async fn deposit(&self, provider: YieldProvider, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let provider = self.provider(provider)?;
        provider.deposit(asset_id, wallet_address, value).await
    }

    pub async fn withdraw(&self, provider: YieldProvider, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let provider = self.provider(provider)?;
        provider.withdraw(asset_id, wallet_address, value).await
    }

    pub async fn positions(&self, provider: YieldProvider, request: &YieldDetailsRequest) -> Result<YieldPosition, YieldError> {
        let provider = self.provider(provider)?;
        provider.positions(request).await
    }

    fn provider(&self, provider: YieldProvider) -> Result<Arc<dyn YieldProviderClient>, YieldError> {
        self.providers
            .iter()
            .find(|candidate| candidate.provider() == provider)
            .cloned()
            .ok_or_else(|| YieldError::new(format!("provider {provider} not found")))
    }
}

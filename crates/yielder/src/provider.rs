use std::sync::Arc;

use async_trait::async_trait;
use primitives::AssetId;

use crate::models::{Yield, YieldDetailsRequest, YieldPosition, YieldProvider, YieldTransaction};
use crate::yo::YieldError;

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

    pub fn is_yield_available(&self, asset_id: &AssetId) -> bool {
        self.providers.iter().any(|provider| !provider.yields(asset_id).is_empty())
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
            .ok_or_else(|| format!("provider {provider} not found").into())
    }
}

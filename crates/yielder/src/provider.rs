use std::sync::Arc;

use async_trait::async_trait;
use primitives::{AssetId, DelegationBase, YieldProvider};

use crate::models::{Yield, YieldDetailsRequest, YieldTransaction};
use crate::yo::YieldError;

#[async_trait]
pub trait YieldProviderClient: Send + Sync {
    fn provider(&self) -> YieldProvider;
    fn yields(&self, asset_id: &AssetId) -> Vec<Yield>;
    async fn deposit(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError>;
    async fn withdraw(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError>;
    async fn positions(&self, request: &YieldDetailsRequest) -> Result<DelegationBase, YieldError>;
    async fn yields_with_apy(&self, asset_id: &AssetId) -> Result<Vec<Yield>, YieldError> {
        Ok(self.yields(asset_id))
    }
}

pub struct Yielder {
    providers: Vec<Arc<dyn YieldProviderClient>>,
}

impl Yielder {
    pub fn new(providers: Vec<Arc<dyn YieldProviderClient>>) -> Self {
        Self { providers }
    }

    pub fn yields_for_asset(&self, asset_id: &AssetId) -> Vec<Yield> {
        self.providers.iter().flat_map(|provider| provider.yields(asset_id)).collect()
    }

    pub async fn yields_for_asset_with_apy(&self, asset_id: &AssetId) -> Result<Vec<Yield>, YieldError> {
        let mut yields = Vec::new();
        for provider in &self.providers {
            yields.extend(provider.yields_with_apy(asset_id).await?);
        }
        yields.sort_by(|a, b| {
            let apy_cmp = b.apy.partial_cmp(&a.apy).unwrap_or(std::cmp::Ordering::Equal);
            apy_cmp.then_with(|| a.name.cmp(&b.name))
        });
        Ok(yields)
    }

    pub async fn deposit(&self, provider: YieldProvider, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let provider = self.get_provider(provider)?;
        provider.deposit(asset_id, wallet_address, value).await
    }

    pub async fn withdraw(&self, provider: YieldProvider, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let provider = self.get_provider(provider)?;
        provider.withdraw(asset_id, wallet_address, value).await
    }

    pub async fn positions(&self, provider: YieldProvider, request: &YieldDetailsRequest) -> Result<DelegationBase, YieldError> {
        let provider = self.get_provider(provider)?;
        provider.positions(request).await
    }

    fn get_provider(&self, provider: YieldProvider) -> Result<Arc<dyn YieldProviderClient>, YieldError> {
        self.providers
            .iter()
            .find(|candidate| candidate.provider() == provider)
            .cloned()
            .ok_or_else(|| format!("provider {provider} not found").into())
    }
}

use std::sync::Arc;

use async_trait::async_trait;
use primitives::{AssetId, Chain, DelegationBase, YieldProvider};

use crate::models::{EarnTransaction, Yield, YieldDetailsRequest};
use crate::yo::YieldError;

#[async_trait]
pub trait YieldProviderClient: Send + Sync {
    fn provider(&self) -> YieldProvider;
    fn yields(&self, asset_id: &AssetId) -> Vec<Yield>;
    fn yields_for_chain(&self, _chain: Chain) -> Vec<Yield> {
        vec![]
    }

    async fn deposit(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<EarnTransaction, YieldError>;
    async fn withdraw(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<EarnTransaction, YieldError>;
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

    pub async fn deposit(&self, provider: YieldProvider, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<EarnTransaction, YieldError> {
        let provider = self.get_provider(provider)?;
        provider.deposit(asset_id, wallet_address, value).await
    }

    pub async fn withdraw(&self, provider: YieldProvider, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<EarnTransaction, YieldError> {
        let provider = self.get_provider(provider)?;
        provider.withdraw(asset_id, wallet_address, value).await
    }

    pub async fn positions(&self, provider: YieldProvider, request: &YieldDetailsRequest) -> Result<DelegationBase, YieldError> {
        let provider = self.get_provider(provider)?;
        provider.positions(request).await
    }

    pub fn yields_for_chain(&self, chain: Chain) -> Vec<Yield> {
        self.providers.iter().flat_map(|p| p.yields_for_chain(chain)).collect()
    }

    pub async fn positions_for_chain(&self, chain: Chain, address: &str) -> Vec<(Yield, DelegationBase)> {
        let futures: Vec<_> = self
            .yields_for_chain(chain)
            .into_iter()
            .map(|y| {
                let address = address.to_string();
                async move {
                    let request = YieldDetailsRequest {
                        asset_id: y.asset_id.clone(),
                        wallet_address: address,
                    };
                    self.positions(y.provider, &request).await.ok().map(|d| (y, d))
                }
            })
            .collect();
        futures::future::join_all(futures).await.into_iter().flatten().collect()
    }

    fn get_provider(&self, provider: YieldProvider) -> Result<Arc<dyn YieldProviderClient>, YieldError> {
        self.providers
            .iter()
            .find(|candidate| candidate.provider() == provider)
            .cloned()
            .ok_or_else(|| format!("provider {provider} not found").into())
    }
}

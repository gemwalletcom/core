use std::sync::Arc;

use alloy_primitives::Address;
use async_trait::async_trait;
use primitives::{AssetId, Chain};

use crate::yo::YieldError;

#[derive(Debug, Clone)]
pub struct Yield {
    pub name: String,
    pub asset: AssetId,
    pub provider: String,
    pub apy: Option<f64>,
}

impl Yield {
    pub fn new(name: impl Into<String>, asset: AssetId, provider: impl Into<String>, apy: Option<f64>) -> Self {
        Self {
            name: name.into(),
            asset,
            provider: provider.into(),
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
pub struct YieldDepositRequest {
    pub asset: AssetId,
    pub wallet_address: String,
    pub receiver_address: Option<String>,
    pub amount: String,
    pub min_shares: Option<String>,
    pub partner_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct YieldWithdrawRequest {
    pub asset: AssetId,
    pub wallet_address: String,
    pub receiver_address: Option<String>,
    pub shares: String,
    pub min_assets: Option<String>,
    pub partner_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct YieldDetailsRequest {
    pub asset: AssetId,
    pub wallet_address: String,
}

#[derive(Debug, Clone)]
pub struct YieldDetails {
    pub asset: AssetId,
    pub provider: String,
    pub share_token: String,
    pub asset_token: String,
    pub share_balance: Option<String>,
    pub asset_balance: Option<String>,
    pub rewards: Option<String>,
}

impl YieldDetails {
    pub fn new(asset: AssetId, provider: impl Into<String>, share_token: Address, asset_token: Address) -> Self {
        Self {
            asset,
            provider: provider.into(),
            share_token: share_token.to_string(),
            asset_token: asset_token.to_string(),
            share_balance: None,
            asset_balance: None,
            rewards: None,
        }
    }
}

#[async_trait]
pub trait YieldProvider: Send + Sync {
    fn protocol(&self) -> &'static str;
    fn yields(&self, asset_id: &AssetId) -> Vec<Yield>;
    async fn deposit(&self, request: &YieldDepositRequest) -> Result<YieldTransaction, YieldError>;
    async fn withdraw(&self, request: &YieldWithdrawRequest) -> Result<YieldTransaction, YieldError>;
    async fn details(&self, request: &YieldDetailsRequest) -> Result<YieldDetails, YieldError>;
}

#[derive(Default)]
pub struct Yielder {
    providers: Vec<Arc<dyn YieldProvider>>,
}

impl Yielder {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    pub fn with_providers(providers: Vec<Arc<dyn YieldProvider>>) -> Self {
        Self { providers }
    }

    pub fn add_provider<P>(&mut self, provider: P)
    where
        P: YieldProvider + 'static,
    {
        self.providers.push(Arc::new(provider));
    }

    pub fn add_provider_arc(&mut self, provider: Arc<dyn YieldProvider>) {
        self.providers.push(provider);
    }

    pub fn yields_for_asset(&self, asset_id: &AssetId) -> Vec<Yield> {
        self.providers.iter().flat_map(|provider| provider.yields(asset_id)).collect()
    }

    pub async fn deposit(&self, protocol: &str, request: &YieldDepositRequest) -> Result<YieldTransaction, YieldError> {
        let provider = self.provider(protocol)?;
        provider.deposit(request).await
    }

    pub async fn withdraw(&self, protocol: &str, request: &YieldWithdrawRequest) -> Result<YieldTransaction, YieldError> {
        let provider = self.provider(protocol)?;
        provider.withdraw(request).await
    }

    pub async fn details(&self, protocol: &str, request: &YieldDetailsRequest) -> Result<YieldDetails, YieldError> {
        let provider = self.provider(protocol)?;
        provider.details(request).await
    }

    fn provider(&self, protocol: &str) -> Result<Arc<dyn YieldProvider>, YieldError> {
        self.providers
            .iter()
            .find(|provider| provider.protocol().eq_ignore_ascii_case(protocol))
            .cloned()
            .ok_or_else(|| YieldError::new(format!("provider {protocol} not found")))
    }
}

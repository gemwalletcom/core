use std::{fmt, str::FromStr};

use alloy_primitives::Address;
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
    pub name: String,
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
    pub fn new(name: impl Into<String>, asset_id: AssetId, provider: YieldProvider, share_token: Address, asset_token: Address) -> Self {
        Self {
            name: name.into(),
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

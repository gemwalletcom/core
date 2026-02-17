use primitives::{AssetId, YieldProvider};

pub use primitives::EarnTransaction;

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
pub struct YieldDetailsRequest {
    pub asset_id: AssetId,
    pub wallet_address: String,
}

pub use primitives::YieldProvider;
pub use primitives::EarnPositionData;
use primitives::{AssetId, Chain, swap::ApprovalData};

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
    pub approval: Option<ApprovalData>,
}

#[derive(Debug, Clone)]
pub struct YieldDetailsRequest {
    pub asset_id: AssetId,
    pub wallet_address: String,
}

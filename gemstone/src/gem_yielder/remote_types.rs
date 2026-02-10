use primitives::{AssetId, YieldProvider};
use yielder::{Yield, EarnTransaction};

use crate::models::swap::GemApprovalData;

pub type GemYieldProvider = YieldProvider;

#[uniffi::remote(Enum)]
pub enum GemYieldProvider {
    Yo,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemEarnTransactionData {
    pub transaction: GemEarnTransaction,
    pub nonce: u64,
    pub chain_id: u64,
    pub gas_limit: String,
}

pub type GemYield = Yield;

#[uniffi::remote(Record)]
pub struct GemYield {
    pub name: String,
    pub asset_id: AssetId,
    pub provider: GemYieldProvider,
    pub apy: Option<f64>,
}

pub type GemEarnTransaction = EarnTransaction;

#[uniffi::remote(Record)]
pub struct GemEarnTransaction {
    pub chain: primitives::Chain,
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: Option<String>,
    pub approval: Option<GemApprovalData>,
}

use primitives::{AssetId, EarnPosition, EarnProvider};
use yielder::{Yield, YieldTransaction};

use crate::models::GemBigInt;
use crate::models::swap::GemApprovalData;
pub use crate::models::transaction::GemEarnAction;

pub type GemEarnProvider = EarnProvider;

#[uniffi::remote(Enum)]
pub enum GemEarnProvider {
    Yo,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemYieldTransactionData {
    pub transaction: GemYieldTransaction,
    pub nonce: u64,
    pub chain_id: u64,
    pub gas_limit: String,
}

pub type GemYield = Yield;

#[uniffi::remote(Record)]
pub struct GemYield {
    pub name: String,
    pub asset_id: AssetId,
    pub provider: GemEarnProvider,
    pub apy: Option<f64>,
}

pub type GemYieldTransaction = YieldTransaction;

#[uniffi::remote(Record)]
pub struct GemYieldTransaction {
    pub chain: primitives::Chain,
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: Option<String>,
    pub approval: Option<GemApprovalData>,
}

pub type GemEarnPosition = EarnPosition;

#[uniffi::remote(Record)]
pub struct GemEarnPosition {
    pub asset_id: AssetId,
    pub provider: GemEarnProvider,
    pub vault_token_address: String,
    pub asset_token_address: String,
    pub vault_balance_value: GemBigInt,
    pub asset_balance_value: GemBigInt,
    pub balance: String,
    pub apy: Option<f64>,
    pub rewards: Option<String>,
}

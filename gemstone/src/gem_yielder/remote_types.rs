use primitives::AssetId;
use yielder::{Yield, YieldPosition, YieldProvider, YieldTransaction};

pub use crate::models::transaction::GemYieldAction;

pub type GemYieldProvider = YieldProvider;

#[uniffi::remote(Enum)]
pub enum GemYieldProvider {
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
    pub provider: GemYieldProvider,
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
}

pub type GemYieldPosition = YieldPosition;

#[uniffi::remote(Record)]
pub struct GemYieldPosition {
    pub name: String,
    pub asset_id: AssetId,
    pub provider: GemYieldProvider,
    pub vault_token_address: String,
    pub asset_token_address: String,
    pub vault_balance_value: Option<String>,
    pub asset_balance_value: Option<String>,
    pub apy: Option<f64>,
    pub rewards: Option<String>,
}

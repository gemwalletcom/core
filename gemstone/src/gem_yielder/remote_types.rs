use primitives::AssetId;
use yielder::{Yield, YieldPosition, YieldProvider, YieldTransaction};

pub type GemYieldProvider = YieldProvider;

#[uniffi::remote(Enum)]
pub enum GemYieldProvider {
    Yo,
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

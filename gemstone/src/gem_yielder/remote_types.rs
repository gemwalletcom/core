use primitives::AssetId;
use yielder::{Yield as CoreYield, YieldPosition as CorePosition, YieldProvider as CoreYieldProvider, YieldTransaction as CoreTransaction};

pub type GemYieldProvider = CoreYieldProvider;

#[uniffi::remote(Enum)]
pub enum GemYieldProvider {
    Yo,
}

pub type GemYield = CoreYield;

#[uniffi::remote(Record)]
pub struct GemYield {
    pub name: String,
    pub asset_id: AssetId,
    pub provider: GemYieldProvider,
    pub apy: Option<f64>,
}

pub type GemYieldTransaction = CoreTransaction;

#[uniffi::remote(Record)]
pub struct GemYieldTransaction {
    pub chain: primitives::Chain,
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: Option<String>,
}

pub type GemYieldPosition = CorePosition;

#[uniffi::remote(Record)]
pub struct GemYieldPosition {
    pub asset_id: AssetId,
    pub provider: GemYieldProvider,
    pub vault_token_address: String,
    pub asset_token_address: String,
    pub vault_balance_value: Option<String>,
    pub asset_balance_value: Option<String>,
    pub apy: Option<f64>,
    pub rewards: Option<String>,
}

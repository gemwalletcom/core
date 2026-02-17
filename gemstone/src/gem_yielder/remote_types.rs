use primitives::{AssetId, YieldProvider};
use yielder::Yield;

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

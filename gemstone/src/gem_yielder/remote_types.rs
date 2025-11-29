use primitives::AssetId;
use yielder::{Yield as CoreYield, YieldDetails as CoreDetails, YieldTransaction as CoreTransaction};

pub type GemYield = CoreYield;

#[uniffi::remote(Record)]
pub struct GemYield {
    pub name: String,
    pub asset: AssetId,
    pub provider: String,
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

pub type GemYieldDetails = CoreDetails;

#[uniffi::remote(Record)]
pub struct GemYieldDetails {
    pub asset: AssetId,
    pub provider: String,
    pub share_token: String,
    pub asset_token: String,
    pub share_balance: Option<String>,
    pub asset_balance: Option<String>,
    pub apy: Option<f64>,
    pub rewards: Option<String>,
}

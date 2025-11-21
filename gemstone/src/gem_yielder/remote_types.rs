use primitives::AssetId;
use yielder::{
    Yield as CoreYield,
    YieldDepositRequest as CoreDepositRequest,
    YieldDetails as CoreDetails,
    YieldDetailsRequest as CoreDetailsRequest,
    YieldTransaction as CoreTransaction,
    YieldWithdrawRequest as CoreWithdrawRequest,
};

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

pub type GemYieldDepositRequest = CoreDepositRequest;

#[uniffi::remote(Record)]
pub struct GemYieldDepositRequest {
    pub asset: AssetId,
    pub wallet_address: String,
    pub receiver_address: Option<String>,
    pub amount: String,
    pub min_shares: Option<String>,
    pub partner_id: Option<u32>,
}

pub type GemYieldWithdrawRequest = CoreWithdrawRequest;

#[uniffi::remote(Record)]
pub struct GemYieldWithdrawRequest {
    pub asset: AssetId,
    pub wallet_address: String,
    pub receiver_address: Option<String>,
    pub shares: String,
    pub min_assets: Option<String>,
    pub partner_id: Option<u32>,
}

pub type GemYieldDetailsRequest = CoreDetailsRequest;

#[uniffi::remote(Record)]
pub struct GemYieldDetailsRequest {
    pub asset: AssetId,
    pub wallet_address: String,
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
    pub rewards: Option<String>,
}

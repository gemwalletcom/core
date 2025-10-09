use primitives::Chain;
use swapper::{SwapperSlippage, config as swap_config};

pub use swap_config::get_swap_config;
pub use swap_config::{Config as SwapConfig, ReferralFee as SwapReferralFee, ReferralFees as SwapReferralFees};

#[uniffi::remote(Record)]
pub struct SwapConfig {
    pub default_slippage: SwapperSlippage,
    pub permit2_expiration: u64,
    pub permit2_sig_deadline: u64,
    pub referral_fee: SwapReferralFees,
    pub high_price_impact_percent: u32,
}

#[uniffi::remote(Record)]
pub struct SwapReferralFees {
    pub evm: SwapReferralFee,
    pub evm_bridge: SwapReferralFee,
    pub solana: SwapReferralFee,
    pub thorchain: SwapReferralFee,
    pub sui: SwapReferralFee,
    pub ton: SwapReferralFee,
    pub tron: SwapReferralFee,
    pub near: SwapReferralFee,
}

#[uniffi::remote(Record)]
pub struct SwapReferralFee {
    pub address: String,
    pub bps: u32,
}

#[uniffi::export]
pub fn get_default_slippage(chain: &Chain) -> SwapperSlippage {
    swap_config::get_default_slippage(chain)
}

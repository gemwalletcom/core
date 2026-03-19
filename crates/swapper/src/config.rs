use crate::{
    SwapperSlippage, SwapperSlippageMode,
    fees::{ReferralFees, default_referral_fees},
};
use primitives::Chain;

pub const DEFAULT_SLIPPAGE_BPS: u32 = 100;
pub const DEFAULT_SWAP_FEE_BPS: u32 = 50;
pub const DEFAULT_CHAINFLIP_FEE_BPS: u32 = 45;
pub const DEFAULT_STABLE_SWAP_REFERRAL_BPS: u32 = 25;

pub const API_BASE_URL: &str = "https://api.gemwallet.com/proxy/swap";

pub fn get_swap_api_url(path: &str) -> String {
    format!("{API_BASE_URL}/{path}")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub default_slippage: SwapperSlippage,
    pub permit2_expiration: u64,
    pub permit2_sig_deadline: u64,
    pub referral_fee: ReferralFees,
    pub high_price_impact_percent: u32,
}

pub fn get_swap_config() -> Config {
    Config {
        default_slippage: SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: SwapperSlippageMode::Exact,
        },
        permit2_expiration: 2_592_000, // 30 days
        permit2_sig_deadline: 1800,    // 30 minutes
        referral_fee: default_referral_fees(),
        high_price_impact_percent: 10,
    }
}

pub fn get_default_slippage(chain: &Chain) -> SwapperSlippage {
    match chain {
        Chain::Solana => SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS * 3,
            mode: SwapperSlippageMode::Exact,
        },
        _ => SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: SwapperSlippageMode::Exact,
        },
    }
}

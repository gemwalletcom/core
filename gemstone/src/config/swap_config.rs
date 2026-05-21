use primitives::Chain;
use swapper::{SwapperSlippage, config as swap_config};

pub use swap_config::{Config as SwapConfig, get_swap_config};

#[uniffi::remote(Record)]
pub struct SwapConfig {
    pub default_slippage: SwapperSlippage,
    pub permit2_expiration: u64,
    pub permit2_sig_deadline: u64,
    pub high_price_impact_percent: u32,
}

#[uniffi::export]
pub fn get_default_slippage(chain: &Chain) -> SwapperSlippage {
    swap_config::get_default_slippage(chain)
}

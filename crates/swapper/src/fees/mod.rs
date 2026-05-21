mod referral;
mod reserve;
mod slippage;

pub use referral::{ReferralFee, ReferralFees, default_referral_address, default_referral_fees};
pub use reserve::{RESERVED_NATIVE_FEES, quote_value_after_reserve, quote_value_after_reserve_by_chain, reserved_tx_fees};
pub use slippage::{BasisPointConvert, apply_slippage_in_bp, bps_to_percent_string};

pub const DEFAULT_SWAP_FEE_BPS: u32 = 50;
pub const DEFAULT_AGGREGATOR_FEE_BPS: u32 = 70;
pub const DEFAULT_CHAINFLIP_FEE_BPS: u32 = 45;
pub const DEFAULT_REFERRER: &str = "gemwallet";

pub(crate) fn is_stablecoin_symbol(symbol: &str) -> bool {
    symbol.to_ascii_uppercase().contains("USD")
}

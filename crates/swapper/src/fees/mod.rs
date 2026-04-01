mod referral;
mod reserve;
mod slippage;

pub use referral::{ReferralFee, ReferralFees, default_referral_fees};
pub use reserve::{RESERVED_NATIVE_FEES, reserved_tx_fees, resolve_max_quote_value};
pub use slippage::{BasisPointConvert, apply_slippage_in_bp};

pub const DEFAULT_SWAP_FEE_BPS: u32 = 50;
pub const DEFAULT_AGGREGATOR_FEE_BPS: u32 = 70;
pub const DEFAULT_CHAINFLIP_FEE_BPS: u32 = 45;
pub const DEFAULT_STABLE_SWAP_REFERRAL_BPS: u32 = 25;
pub const DEFAULT_REFERRER: &str = "gemwallet";

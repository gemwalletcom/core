pub static DEFAULT_MAX_LEVERAGE: u32 = 10;
pub static DEFAULT_FEE_BPS: u32 = 10; // 0.1%
pub static DEFAULT_MIN_POSITION_SIZE_USD: u64 = 10;
pub static DEFAULT_MAX_BUILDER_FEE_BPS: u32 = 45; // 0.045%

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct PerpetualConfig {
    pub default_max_leverage: u32,
    pub fee_bps: u32,
    pub min_position_size_usd: u64,
    pub builder_address: String,
    pub referral_code: String,
    pub max_builder_fee_bps: u32,
    pub referral_fee: PerpetualReferralFee,
}

#[derive(uniffi::Record, Default, Debug, Clone, PartialEq)]
pub struct PerpetualReferralFee {
    pub address: String,
    pub bps: u32,
}

pub fn get_perpetual_config() -> PerpetualConfig {
    PerpetualConfig {
        default_max_leverage: DEFAULT_MAX_LEVERAGE,
        fee_bps: DEFAULT_FEE_BPS,
        min_position_size_usd: DEFAULT_MIN_POSITION_SIZE_USD,
        builder_address: "0x0d9dab1a248f63b0a48965ba8435e4de7497a3dc".into(),
        referral_code: "GEMWALLET".into(),
        max_builder_fee_bps: DEFAULT_MAX_BUILDER_FEE_BPS,
        referral_fee: PerpetualReferralFee {
            address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            bps: DEFAULT_FEE_BPS,
        },
    }
}
#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct PerpetualConfig {
    pub builder_address: String,
    pub referral_code: String,
    pub max_builder_fee_bps: u32,
}

pub fn get_perpetual_config() -> PerpetualConfig {
    PerpetualConfig {
        builder_address: "0x0d9dab1a248f63b0a48965ba8435e4de7497a3dc".into(),
        referral_code: "GEMWALLET".into(),
        max_builder_fee_bps: 45,
    }
}

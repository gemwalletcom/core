#[derive(Debug, Clone, PartialEq)]
pub struct HypercoreConfig {
    pub builder_address: String,
    pub referral_code: String,
    pub max_builder_fee_bps: u32,
}

impl Default for HypercoreConfig {
    fn default() -> Self {
        Self {
            builder_address: "0x0d9dab1a248f63b0a48965ba8435e4de7497a3dc".to_string(),
            referral_code: "GEMWALLET".to_string(),
            max_builder_fee_bps: 45,
        }
    }
}

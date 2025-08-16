// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.
#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperUpdateLeverage {
    pub r#type: String,
    pub asset: u32,
    #[serde(rename = "isCross")]
    pub is_cross: bool,
    pub leverage: u64,
}

impl HyperUpdateLeverage {
    pub fn new(asset: u32, is_cross: bool, leverage: u64) -> Self {
        Self {
            r#type: "updateLeverage".to_string(),
            asset,
            is_cross,
            leverage,
        }
    }
}

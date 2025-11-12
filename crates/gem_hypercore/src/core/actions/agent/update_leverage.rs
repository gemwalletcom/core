// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.
#[derive(Clone, serde::Serialize)]
pub struct UpdateLeverage {
    pub r#type: String,
    pub asset: u32,
    #[serde(rename = "isCross")]
    pub is_cross: bool,
    pub leverage: u8,
}

impl UpdateLeverage {
    pub fn new(asset: u32, is_cross: bool, leverage: u8) -> Self {
        Self {
            r#type: "updateLeverage".to_string(),
            asset,
            is_cross,
            leverage,
        }
    }
}

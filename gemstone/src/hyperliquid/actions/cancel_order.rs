// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperCancel {
    pub cancels: Vec<HyperCancelOrder>,
    pub r#type: String,
}

impl HyperCancel {
    pub fn new(cancels: Vec<HyperCancelOrder>) -> Self {
        Self {
            cancels,
            r#type: "cancel".to_string(),
        }
    }
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperCancelOrder {
    #[serde(rename = "a")]
    pub asset: u32,
    #[serde(rename = "o")]
    pub order_id: u64,
}

impl HyperCancelOrder {
    pub fn new(asset: u32, order_id: u64) -> Self {
        Self { asset, order_id }
    }
}

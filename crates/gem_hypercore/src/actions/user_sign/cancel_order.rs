// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.

#[derive(Clone, serde::Serialize)]
pub struct Cancel {
    pub cancels: Vec<CancelOrder>,
    pub r#type: String,
}

impl Cancel {
    pub fn new(cancels: Vec<CancelOrder>) -> Self {
        Self {
            cancels,
            r#type: "cancel".to_string(),
        }
    }
}

#[derive(Clone, serde::Serialize)]
pub struct CancelOrder {
    #[serde(rename = "a")]
    pub asset: u32,
    #[serde(rename = "o")]
    pub order_id: u64,
}

impl CancelOrder {
    pub fn new(asset: u32, order_id: u64) -> Self {
        Self { asset, order_id }
    }
}

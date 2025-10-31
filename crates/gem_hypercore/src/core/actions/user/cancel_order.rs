// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.

#[derive(Clone, serde::Serialize)]
pub struct Cancel {
    pub r#type: String,
    pub cancels: Vec<CancelOrder>,
}

impl Cancel {
    pub fn new(cancels: Vec<CancelOrder>) -> Self {
        Self {
            r#type: "cancel".to_string(),
            cancels,
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

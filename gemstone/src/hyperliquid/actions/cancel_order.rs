#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperCancelOrder {
    #[serde(rename = "type")]
    pub action_type: String,
    pub cancels: Vec<HyperCancelItem>,
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperCancelItem {
    #[serde(rename = "a")]
    pub asset: u32,
    #[serde(rename = "o")]
    pub order_id: u32,
}

impl HyperCancelOrder {
    pub fn new(cancels: Vec<HyperCancelItem>) -> Self {
        Self {
            action_type: "cancel".to_string(),
            cancels,
        }
    }
}

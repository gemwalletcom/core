use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Webhook {
    pub order_id: String,
}
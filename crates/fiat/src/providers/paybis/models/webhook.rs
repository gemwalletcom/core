use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhook {
    pub transaction_id: String,
}

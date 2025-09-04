use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhook {
    pub data: PaybisWebhookData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookData {
    pub transaction: PaybisWebhookTransaction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookTransaction {
    pub invoice: String,
}

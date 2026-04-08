use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhook<T> {
    pub event: String,
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisWebhookData {
    pub partner_transaction_id: Option<String>,
    pub quote: PaybisWebhookQuote,
    pub transaction: PaybisTransaction,
    pub amount_from: PaybisAmount,
    pub amount_to: PaybisAmount,
    pub payout: Option<PaybisPayout>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisPayout {
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisTransaction {
    pub invoice: String,
    pub status: String,
    pub flow: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisWebhookQuote {
    pub quote_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisAmount {
    pub amount: String,
    pub currency: String,
}

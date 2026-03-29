use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhook<T> {
    pub event: String,
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisWebhookData {
    pub request_id: String,
    pub partner_transaction_id: Option<String>,
    pub quote: PaybisWebhookQuote,
    pub transaction: PaybisTransaction,
    pub amount_from: PaybisAmount,
    pub amount_to: PaybisAmount,
    pub payment: Option<PaybisPayment>,
    pub payout: Option<PaybisPayout>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisPayment {
    pub card: Option<PaybisCard>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisCard {
    pub billing_address: PaybisBillingAddress,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisBillingAddress {
    pub country: PaybisCountry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisCountry {
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisPayout {
    pub transaction_hash: Option<String>,
    pub destination_wallet_address: Option<String>,
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

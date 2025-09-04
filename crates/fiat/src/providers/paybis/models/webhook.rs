use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhook {
    pub data: PaybisWebhookData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookData {
    pub transaction: PaybisWebhookTransaction,
    #[serde(rename = "amountFrom")]
    pub amount_from: PaybisWebhookAmount,
    #[serde(rename = "amountTo")]
    pub amount_to: PaybisWebhookAmount,
    pub payment: Option<PaybisWebhookPayment>,
    pub payout: Option<PaybisWebhookPayout>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookPayment {
    pub card: Option<PaybisWebhookCard>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookCard {
    #[serde(rename = "billingAddress")]
    pub billing_address: PaybisWebhookBillingAddress,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookBillingAddress {
    pub country: PaybisWebhookCountry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookCountry {
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookPayout {
    pub transaction_hash: Option<String>,
    #[serde(rename = "destinationWalletAddress")]
    pub destination_wallet_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookTransaction {
    pub invoice: String,
    pub status: String,
    pub flow: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookAmount {
    pub amount: String,
    pub currency: String,
}

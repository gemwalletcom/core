use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisWebhookData {
    pub transaction: PaybisTransaction,
    #[serde(rename = "amountFrom")]
    pub amount_from: PaybisAmount,
    #[serde(rename = "amountTo")]
    pub amount_to: PaybisAmount,
    pub payment: Option<PaybisPayment>,
    pub payout: Option<PaybisPayout>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisPayment {
    pub card: Option<PaybisCard>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisCard {
    #[serde(rename = "billingAddress")]
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
pub struct PaybisPayout {
    pub transaction_hash: Option<String>,
    #[serde(rename = "destinationWalletAddress")]
    pub destination_wallet_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisTransaction {
    pub invoice: String,
    pub status: String,
    pub flow: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisAmount {
    pub amount: String,
    pub currency: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisQuoteRequest {
    pub currency_code_from: String,
    pub currency_code_to: String,
    pub amount_from: String, // Paybis expects amount as a string
                             // pub amount_to: Option<String>, // Optional: only one of amount_from or amount_to should be provided
                             // pub payment_method: Option<String>,
                             // pub ip_address: Option<String>,
                             // pub country_code: Option<String>,
                             // pub state_code: Option<String>,
                             // pub wallet_address: Option<String>,
                             // pub wallet_address_tag: Option<String>,
                             // pub redirect_url: Option<String>,
                             // pub partner_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisQuoteResponse {
    // pub id: String,
    // pub payment_method: String,
    pub currency_code_from: String,
    pub amount_from: String,
    pub currency_code_to: String,
    pub amount_to: String,
    // pub rate: String,
    // pub fees: PaybisFees,
    pub redirect_url: Option<String>, // This can be null according to docs if not applicable
                                      // pub expires_at: String,
}

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct PaybisFees {
//     pub network_fee: String,
//     pub processing_fee: String,
//     pub total_fee: String,
// }

// payment_method: Option<String>,

// Note: The PaybisFees struct below was what we originally had for the quote response.
// If the actual quote response contains a nested 'fees' object, we might need to reinstate and adjust it.
// For now, keeping it commented out as the current PaybisQuoteResponse flattens fee-related fields or omits them.

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaybisAssetPair {
    pub currency_code_from: String,
    pub currency_code_to: String,
    // Potentially other fields like min/max amounts, networks, etc.
    // We will adjust based on actual API response if needed.
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaybisAmount {
    pub currency: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaybisSuccessfulPayoutEvent {
    pub event_id: String,
    pub transaction_id: String,
    pub digital_amount_sent: PaybisAmount,
    pub blockchain_txn_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaybisCryptoPayoutErrorEvent {
    pub event_id: String,
    pub event_type: String, // e.g., "TransactionCryptoPayoutError"
    pub transaction_id: String,
    pub invoice: Option<String>,
    pub status: String, // e.g., "Rejected"
    pub amount_sent: PaybisAmount, 
    pub reason: Option<String>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PaybisWebhookPayload {
    Success(PaybisSuccessfulPayoutEvent),
    Error(PaybisCryptoPayoutErrorEvent),
}

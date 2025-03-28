use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct QuoteBuy {
    #[serde(rename = "CARD_PAYMENT")]
    pub card_payment: QuoteData,
    pub asset: QuoteAsset,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QuoteSell {
    #[serde(rename = "CARD")]
    pub card_payment: QuoteData,
    pub asset: QuoteAsset,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteData {
    pub crypto_amount: String,
    pub fiat_value: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteAsset {
    pub symbol: String,
    pub chain: String,
    pub decimals: u32,
    pub address: Option<String>,
    pub enabled: Option<bool>,
}

impl QuoteAsset {
    pub fn token_id(&self) -> Option<String> {
        if let Some(address) = &self.address {
            if address.is_empty() {
                None
            } else {
                Some(address.clone())
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct QuoteAssets {
    pub assets: Vec<QuoteAsset>,
}

impl QuoteAsset {
    pub fn crypto_asset_symbol(&self) -> String {
        format!("{}_{}", self.chain, self.symbol)
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub crypto_asset_symbol: String,
    pub fiat_currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fiat_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_amount: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    #[serde(rename = "type")]
    pub webhook_type: String,
    pub purchase: WebhookPurchase,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPurchase {
    pub purchase_view_token: String,
    pub fiat_currency: String,
    pub fiat_value: f64,
    pub applied_fee: f64,
    pub base_ramp_fee: f64,
    pub host_fee_cut: f64,
    pub network_fee: f64,
    pub asset: QuoteAsset,
    pub receiver_address: Option<String>,
    pub final_tx_hash: Option<String>,
    pub status: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetOnrampRequest {
    pub destination_chain: String,
    pub destination_asset: String,
    pub recipient_address: String,
    pub amount: String,
    pub amount_mode: String,
    pub affiliate_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetOnrampResponse {
    pub order_id: String,
    pub payment_links: FlashnetPaymentLinks,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetPaymentLinks {
    pub cash_app: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetRoutesResponse {
    pub routes: Vec<FlashnetRoute>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetEstimateResponse {
    pub estimated_out: String,
    pub fee_bps: u32,
    pub app_fees: Vec<FlashnetEstimateAppFee>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetEstimateAppFee {
    pub affiliate_id: String,
    pub fee_bps: u32,
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetRoute {
    pub source_chain: String,
    pub source_asset: String,
    pub destination: FlashnetRouteAsset,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetRouteAsset {
    pub chain: String,
    pub asset: String,
    pub contract_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetWebhookPayload {
    pub event: String,
    pub data: FlashnetWebhookData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetWebhookData {
    pub id: String,
    pub status: Option<String>,
    pub amount_out: Option<String>,
    pub destination: Option<FlashnetDestination>,
    pub payment_intent: Option<FlashnetPaymentIntent>,
}

impl FlashnetWebhookData {
    pub fn into_order(self) -> Option<FlashnetOrder> {
        let status = self.status?;

        Some(FlashnetOrder {
            id: self.id,
            status,
            amount_out: self.amount_out,
            destination: self.destination,
            payment_intent: self.payment_intent,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetOrder {
    pub id: String,
    pub status: String,
    pub amount_out: Option<String>,
    pub destination: Option<FlashnetDestination>,
    pub payment_intent: Option<FlashnetPaymentIntent>,
}

impl FlashnetOrder {
    pub fn destination_tx_hash(&self) -> Option<&str> {
        self.destination.as_ref().and_then(|destination| destination.tx_hash.as_deref())
    }

    pub fn effective_amount_out(&self) -> Option<&str> {
        self.amount_out.as_deref().or(self.payment_intent.as_ref().and_then(|pi| pi.target_amount_out.as_deref()))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetDestination {
    pub tx_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetPaymentIntent {
    pub target_amount_out: Option<String>,
}

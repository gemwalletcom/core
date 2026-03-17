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
    pub payment_links: FlashnetPaymentLinks,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetPaymentLinks {
    pub cash_app: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetStatusResponse {
    pub order: FlashnetOrder,
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
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetOrder {
    pub id: String,
    pub status: String,
    pub destination_chain: Option<String>,
    pub destination_asset: Option<String>,
    pub recipient_address: Option<String>,
    pub amount_out: Option<String>,
    pub destination: Option<FlashnetDestination>,
}

impl FlashnetOrder {
    pub fn destination_chain(&self) -> Option<&str> {
        self.destination_chain
            .as_deref()
            .or(self.destination.as_ref().and_then(|destination| destination.chain.as_deref()))
    }

    pub fn destination_asset(&self) -> Option<&str> {
        self.destination_asset
            .as_deref()
            .or(self.destination.as_ref().and_then(|destination| destination.asset.as_deref()))
    }

    pub fn recipient_address(&self) -> Option<&str> {
        self.recipient_address
            .as_deref()
            .or(self.destination.as_ref().and_then(|destination| destination.address.as_deref()))
    }

    pub fn destination_tx_hash(&self) -> Option<&str> {
        self.destination.as_ref().and_then(|destination| destination.tx_hash.as_deref())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashnetDestination {
    pub chain: Option<String>,
    pub asset: Option<String>,
    pub address: Option<String>,
    pub tx_hash: Option<String>,
}

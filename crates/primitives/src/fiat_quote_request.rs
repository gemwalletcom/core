use crate::{AssetId, FiatQuoteType};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteRequest {
    #[typeshare(skip)]
    pub asset_id: AssetId,
    #[serde(rename = "type")]
    #[typeshare(skip)]
    pub quote_type: FiatQuoteType,
    pub amount: f64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[typeshare(skip)]
    pub provider_id: Option<String>,
    #[typeshare(skip)]
    pub ip_address: String,
}

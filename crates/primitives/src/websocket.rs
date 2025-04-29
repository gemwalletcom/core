use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, AssetPrice, FiatRate};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[typeshare(swift = "Sendable")]
pub enum WebSocketPriceActionType {
    Subscribe,
    Add,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct WebSocketPriceAction {
    pub action: WebSocketPriceActionType,
    #[serde(default)]
    pub assets: Vec<AssetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct WebSocketPricePayload {
    pub prices: Vec<AssetPrice>,
    pub rates: Vec<FiatRate>,
}

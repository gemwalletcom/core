use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, AssetPrice, Chain, FiatRate, WalletId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[typeshare(swift = "Sendable")]
pub enum StreamMessage {
    SubscribePrices {
        #[serde(default)]
        assets: Vec<AssetId>,
    },
    UnsubscribePrices {
        #[serde(default)]
        assets: Vec<AssetId>,
    },
    AddPrices {
        #[serde(default)]
        assets: Vec<AssetId>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct StreamBalanceUpdate {
    pub wallet_id: WalletId,
    pub chain: Chain,
    pub address: String,
    pub asset_id: AssetId,
}

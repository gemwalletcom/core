use std::collections::HashMap;

use primitives::PerpetualPosition;
use primitives::chart::ChartCandleUpdate;
use primitives::perpetual::PerpetualBalance;
use serde::Deserialize;

use super::candlestick::Candlestick;
use super::order::OpenOrder;
use super::position::AssetPositions;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "channel", content = "data")]
pub enum RawSocketMessage {
    #[serde(rename = "clearinghouseState")]
    ClearinghouseState(ClearinghouseStateData),

    #[serde(rename = "openOrders")]
    OpenOrders(OpenOrdersData),

    #[serde(rename = "candle")]
    Candle(Candlestick),

    #[serde(rename = "allMids")]
    AllMids(AllMidsData),

    #[serde(rename = "subscriptionResponse")]
    SubscriptionResponse(SubscriptionResponseData),

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllMidsData {
    pub mids: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearinghouseStateData {
    pub clearinghouse_state: AssetPositions,
    pub user: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenOrdersData {
    pub user: String,
    pub orders: Vec<OpenOrder>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionResponseData {
    pub subscription: Subscription,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Subscription {
    #[serde(rename = "type")]
    pub subscription_type: String,
}

#[derive(Debug)]
pub struct PositionsDiff {
    pub delete_position_ids: Vec<String>,
    pub positions: Vec<PerpetualPosition>,
}

#[derive(Debug)]
pub enum HyperliquidSocketMessage {
    ClearinghouseState { balance: PerpetualBalance, positions: Vec<PerpetualPosition> },
    OpenOrders { orders: Vec<OpenOrder> },
    Candle { candle: ChartCandleUpdate },
    AllMids { prices: HashMap<String, f64> },
    SubscriptionResponse { subscription_type: String },
    Unknown,
}

use serde::Deserialize;

use super::order::OpenOrder;
use super::position::AssetPositions;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebSocketChannel {
    ClearinghouseState,
    OpenOrders,
    SubscriptionResponse,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketMessage {
    pub channel: WebSocketChannel,
    pub data: serde_json::Value,
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
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    #[serde(rename = "type")]
    pub subscription_type: String,
}

use std::collections::HashMap;

use primitives::PerpetualPosition;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use super::order::OpenOrder;
use super::position::AssetPositions;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebSocketChannel {
    ClearinghouseState,
    OpenOrders,
    Candle,
    AllMids,
    SubscriptionResponse,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllMidsData {
    pub mids: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketMessage<T> {
    pub channel: WebSocketChannel,
    pub data: T,
}

impl<T: DeserializeOwned> WebSocketMessage<T> {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
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

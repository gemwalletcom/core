use primitives::perpetual::PerpetualPositionsSummary;

use crate::models::{
    order::OpenOrder,
    websocket::{ClearinghouseStateData, OpenOrdersData, SubscriptionResponseData, WebSocketChannel, WebSocketMessage},
};

use super::perpetual_mapper::map_positions;

#[derive(Debug)]
pub struct ClearinghouseResult {
    pub user: String,
    pub summary: PerpetualPositionsSummary,
}

#[derive(Debug)]
pub struct OpenOrdersResult {
    pub user: String,
    pub orders: Vec<OpenOrder>,
}

pub fn parse_channel(json: &str) -> Result<WebSocketChannel, serde_json::Error> {
    let message: WebSocketMessage = serde_json::from_str(json)?;
    Ok(message.channel)
}

pub fn parse_clearinghouse_state(json: &str) -> Result<ClearinghouseResult, serde_json::Error> {
    let message: WebSocketMessage = serde_json::from_str(json)?;
    let data: ClearinghouseStateData = serde_json::from_value(message.data)?;

    let summary = map_positions(data.clearinghouse_state, data.user.clone(), &[]);

    Ok(ClearinghouseResult { user: data.user, summary })
}

pub fn parse_open_orders(json: &str) -> Result<OpenOrdersResult, serde_json::Error> {
    let message: WebSocketMessage = serde_json::from_str(json)?;
    let data: OpenOrdersData = serde_json::from_value(message.data)?;

    Ok(OpenOrdersResult {
        user: data.user,
        orders: data.orders,
    })
}

pub fn parse_subscription_response(json: &str) -> Result<String, serde_json::Error> {
    let message: WebSocketMessage = serde_json::from_str(json)?;
    let data: SubscriptionResponseData = serde_json::from_value(message.data)?;
    Ok(data.subscription.subscription_type)
}

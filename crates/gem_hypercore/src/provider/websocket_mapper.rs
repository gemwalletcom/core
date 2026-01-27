use std::collections::HashMap;

use primitives::chart::ChartCandleStick;
use primitives::perpetual::PerpetualPositionsSummary;
use serde::de::DeserializeOwned;

use crate::models::{
    candlestick::Candlestick,
    order::OpenOrder,
    websocket::{AllMidsData, ClearinghouseStateData, OpenOrdersData, SubscriptionResponseData, WebSocketChannel, WebSocketMessage},
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

fn parse_data<T: DeserializeOwned>(json: &str) -> Result<T, serde_json::Error> {
    let message: WebSocketMessage = serde_json::from_str(json)?;
    serde_json::from_value(message.data)
}

pub fn parse_channel(json: &str) -> Result<WebSocketChannel, serde_json::Error> {
    let message: WebSocketMessage = serde_json::from_str(json)?;
    Ok(message.channel)
}

pub fn parse_clearinghouse_state(json: &str) -> Result<ClearinghouseResult, serde_json::Error> {
    let data: ClearinghouseStateData = parse_data(json)?;
    let summary = map_positions(data.clearinghouse_state, data.user.clone(), &[]);
    Ok(ClearinghouseResult { user: data.user, summary })
}

pub fn parse_open_orders(json: &str) -> Result<OpenOrdersResult, serde_json::Error> {
    let data: OpenOrdersData = parse_data(json)?;
    Ok(OpenOrdersResult { user: data.user, orders: data.orders })
}

pub fn parse_subscription_response(json: &str) -> Result<String, serde_json::Error> {
    let data: SubscriptionResponseData = parse_data(json)?;
    Ok(data.subscription.subscription_type)
}

pub fn parse_candle(json: &str) -> Result<ChartCandleStick, serde_json::Error> {
    let candlestick: Candlestick = parse_data(json)?;
    Ok(candlestick.into())
}

pub fn parse_all_mids(json: &str) -> Result<HashMap<String, f64>, serde_json::Error> {
    let data: AllMidsData = parse_data(json)?;
    Ok(data.mids.into_iter().filter_map(|(k, v)| v.parse::<f64>().ok().map(|p| (k, p))).collect())
}

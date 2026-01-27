use std::collections::{HashMap, HashSet};

use primitives::chart::ChartCandleStick;
use primitives::perpetual::PerpetualPositionsSummary;
use primitives::{AssetId, PerpetualPosition};

use crate::models::{
    candlestick::Candlestick,
    order::OpenOrder,
    websocket::{AllMidsData, ClearinghouseStateData, OpenOrdersData, PositionsDiff, SubscriptionResponseData, WebSocketChannel, WebSocketMessage},
};

use super::perpetual_mapper::{map_positions, map_tp_sl_from_orders};

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
    Ok(WebSocketMessage::<serde_json::Value>::parse(json)?.channel)
}

pub fn parse_clearinghouse_state(json: &str) -> Result<ClearinghouseResult, serde_json::Error> {
    let data = WebSocketMessage::<ClearinghouseStateData>::parse(json)?.data;
    let summary = map_positions(data.clearinghouse_state, data.user.clone(), &[]);
    Ok(ClearinghouseResult { user: data.user, summary })
}

pub fn parse_open_orders(json: &str) -> Result<OpenOrdersResult, serde_json::Error> {
    let data = WebSocketMessage::<OpenOrdersData>::parse(json)?.data;
    Ok(OpenOrdersResult { user: data.user, orders: data.orders })
}

pub fn parse_subscription_response(json: &str) -> Result<String, serde_json::Error> {
    let data = WebSocketMessage::<SubscriptionResponseData>::parse(json)?.data;
    Ok(data.subscription.subscription_type)
}

pub fn parse_candle(json: &str) -> Result<ChartCandleStick, serde_json::Error> {
    Ok(WebSocketMessage::<Candlestick>::parse(json)?.data.into())
}

pub fn parse_all_mids(json: &str) -> Result<HashMap<String, f64>, serde_json::Error> {
    let data = WebSocketMessage::<AllMidsData>::parse(json)?.data;
    Ok(data.mids.into_iter().filter_map(|(k, v)| v.parse::<f64>().ok().map(|p| (k, p))).collect())
}

pub fn diff_clearinghouse_positions(new_positions: Vec<PerpetualPosition>, existing_positions: Vec<PerpetualPosition>) -> PositionsDiff {
    let existing_map: HashMap<&str, &PerpetualPosition> = existing_positions.iter().map(|p| (p.id.as_str(), p)).collect();

    let positions: Vec<PerpetualPosition> = new_positions
        .into_iter()
        .map(|pos| match existing_map.get(pos.id.as_str()) {
            Some(existing) => PerpetualPosition {
                take_profit: existing.take_profit.clone(),
                stop_loss: existing.stop_loss.clone(),
                ..pos
            },
            None => pos,
        })
        .collect();

    let new_ids: HashSet<&str> = positions.iter().map(|p| p.id.as_str()).collect();
    let delete_position_ids: Vec<String> = existing_positions.iter().filter(|p| !new_ids.contains(p.id.as_str())).map(|p| p.id.clone()).collect();

    PositionsDiff { delete_position_ids, positions }
}

pub fn diff_open_orders_positions(orders: &[OpenOrder], existing_positions: Vec<PerpetualPosition>) -> PositionsDiff {
    let positions: Vec<PerpetualPosition> = existing_positions
        .into_iter()
        .filter_map(|pos| {
            let coin = pos.asset_id.token_id.as_ref().and_then(|t| AssetId::decode_token_id(t).into_iter().nth(1))?;
            let (take_profit, stop_loss) = map_tp_sl_from_orders(orders, &coin);

            if pos.take_profit != take_profit || pos.stop_loss != stop_loss {
                Some(PerpetualPosition { take_profit, stop_loss, ..pos })
            } else {
                None
            }
        })
        .collect();

    PositionsDiff {
        delete_position_ids: vec![],
        positions,
    }
}

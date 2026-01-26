use std::collections::{HashMap, HashSet};

use gem_hypercore::{
    models::websocket::WebSocketChannel,
    perpetual_formatter::PerpetualFormatter,
    provider::{
        perpetual_mapper::map_tp_sl_from_orders,
        websocket_mapper::{parse_candle, parse_channel, parse_clearinghouse_state, parse_open_orders, parse_subscription_response},
    },
};
use primitives::{AssetId, PerpetualPosition, PerpetualProvider};

use crate::models::perpetual::{GemHyperliquidOpenOrder, GemHyperliquidSocketMessage, GemPositionsDiff};

#[derive(Debug, uniffi::Object)]
pub struct Perpetual {
    provider: PerpetualProvider,
}

#[uniffi::export]
impl Perpetual {
    #[uniffi::constructor]
    pub fn new(provider: PerpetualProvider) -> Self {
        Self { provider }
    }

    pub fn minimum_order_usd_amount(&self, price: f64, decimals: i32, leverage: u8) -> u64 {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::minimum_order_usd_amount(price, decimals, leverage),
        }
    }

    pub fn format_price(&self, price: f64, decimals: i32) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_price(price, decimals),
        }
    }

    pub fn format_size(&self, size: f64, decimals: i32) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_size(size, decimals),
        }
    }
}

#[derive(Debug, uniffi::Object)]
pub struct Hyperliquid {}

impl Default for Hyperliquid {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl Hyperliquid {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_websocket_data(&self, data: Vec<u8>) -> Result<GemHyperliquidSocketMessage, crate::GemstoneError> {
        let json = String::from_utf8(data)?;
        let channel = parse_channel(&json)?;

        match channel {
            WebSocketChannel::ClearinghouseState => {
                let result = parse_clearinghouse_state(&json)?;
                Ok(GemHyperliquidSocketMessage::ClearinghouseState {
                    balance: result.summary.balance,
                    positions: result.summary.positions,
                })
            }
            WebSocketChannel::OpenOrders => {
                let result = parse_open_orders(&json)?;
                Ok(GemHyperliquidSocketMessage::OpenOrders { orders: result.orders })
            }
            WebSocketChannel::Candle => {
                let candle = parse_candle(&json)?;
                Ok(GemHyperliquidSocketMessage::Candle { candle })
            }
            WebSocketChannel::SubscriptionResponse => {
                let subscription_type = parse_subscription_response(&json)?;
                Ok(GemHyperliquidSocketMessage::SubscriptionResponse { subscription_type })
            }
            WebSocketChannel::Unknown => Ok(GemHyperliquidSocketMessage::Unknown),
        }
    }

    pub fn diff_clearinghouse_positions(&self, new_positions: Vec<PerpetualPosition>, existing_positions: Vec<PerpetualPosition>) -> GemPositionsDiff {
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

        GemPositionsDiff { delete_position_ids, positions }
    }

    pub fn diff_open_orders_positions(&self, orders: Vec<GemHyperliquidOpenOrder>, existing_positions: Vec<PerpetualPosition>) -> GemPositionsDiff {
        let positions: Vec<PerpetualPosition> = existing_positions
            .into_iter()
            .filter_map(|pos| {
                let coin = pos.asset_id.token_id.as_ref().and_then(|t| AssetId::decode_token_id(t).into_iter().nth(1))?;
                let (take_profit, stop_loss) = map_tp_sl_from_orders(&orders, &coin);

                if pos.take_profit != take_profit || pos.stop_loss != stop_loss {
                    Some(PerpetualPosition { take_profit, stop_loss, ..pos })
                } else {
                    None
                }
            })
            .collect();

        GemPositionsDiff {
            delete_position_ids: vec![],
            positions,
        }
    }
}

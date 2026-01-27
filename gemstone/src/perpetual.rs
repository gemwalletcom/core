use gem_hypercore::{
    models::websocket::WebSocketChannel,
    perpetual_formatter::PerpetualFormatter,
    provider::websocket_mapper::{
        diff_clearinghouse_positions, diff_open_orders_positions, parse_all_mids, parse_candle, parse_channel, parse_clearinghouse_state,
        parse_open_orders, parse_subscription_response,
    },
};
use primitives::{PerpetualPosition, PerpetualProvider};

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
            WebSocketChannel::AllMids => {
                let prices = parse_all_mids(&json)?;
                Ok(GemHyperliquidSocketMessage::AllMids { prices })
            }
            WebSocketChannel::SubscriptionResponse => {
                let subscription_type = parse_subscription_response(&json)?;
                Ok(GemHyperliquidSocketMessage::SubscriptionResponse { subscription_type })
            }
            WebSocketChannel::Unknown => Ok(GemHyperliquidSocketMessage::Unknown),
        }
    }

    pub fn diff_clearinghouse_positions(&self, new_positions: Vec<PerpetualPosition>, existing_positions: Vec<PerpetualPosition>) -> GemPositionsDiff {
        diff_clearinghouse_positions(new_positions, existing_positions)
    }

    pub fn diff_open_orders_positions(&self, orders: Vec<GemHyperliquidOpenOrder>, existing_positions: Vec<PerpetualPosition>) -> GemPositionsDiff {
        diff_open_orders_positions(&orders, existing_positions)
    }
}

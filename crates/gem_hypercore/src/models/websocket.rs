use std::collections::HashMap;

use primitives::chart::ChartCandleUpdate;
use primitives::perpetual::PerpetualBalance;
use primitives::{PerpetualMarketData, PerpetualPosition};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_f64_from_str, deserialize_option_f64_from_str};

use super::candlestick::Candlestick;
use super::order::OpenOrder;
use super::position::AssetPositions;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "channel", content = "data")]
pub enum RawSocketMessage {
    #[serde(rename = "clearinghouseState")]
    AccountState(AccountStateData),

    #[serde(rename = "openOrders")]
    OpenOrders(OpenOrdersData),

    #[serde(rename = "candle")]
    Candle(Candlestick),

    #[serde(rename = "activeAssetCtx")]
    MarketData(ActiveAssetCtxData),

    #[serde(rename = "allMids")]
    MarketPrices(AllMidsData),

    #[serde(rename = "subscriptionResponse")]
    SubscriptionResponse(SubscriptionResponseData),

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum HyperliquidMethod {
    Subscribe,
    Unsubscribe,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "type")]
pub enum HyperliquidSubscription {
    #[serde(rename = "clearinghouseState")]
    AccountState {
        #[serde(rename = "user")]
        address: String,
    },

    #[serde(rename = "openOrders")]
    OpenOrders {
        #[serde(rename = "user")]
        address: String,
    },

    #[serde(rename = "candle")]
    Candle {
        #[serde(rename = "coin")]
        symbol: String,
        interval: String,
    },

    #[serde(rename = "activeAssetCtx")]
    MarketData {
        #[serde(rename = "coin")]
        symbol: String,
    },

    #[serde(rename = "allMids")]
    MarketPrices,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HyperliquidRequest {
    pub method: HyperliquidMethod,
    pub subscription: HyperliquidSubscription,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllMidsData {
    pub mids: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActiveAssetCtxData {
    #[serde(rename = "coin")]
    pub symbol: String,
    pub ctx: ActiveAssetCtx,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAssetCtx {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub day_ntl_vlm: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub prev_day_px: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub mark_px: f64,
    #[serde(default, deserialize_with = "deserialize_option_f64_from_str")]
    pub mid_px: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub funding: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub open_interest: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountStateData {
    pub clearinghouse_state: AssetPositions,
    #[serde(rename = "user")]
    pub address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenOrdersData {
    #[serde(rename = "user")]
    pub address: String,
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
    AccountState { balance: PerpetualBalance, positions: Vec<PerpetualPosition> },
    OpenOrders { orders: Vec<OpenOrder> },
    Candle { candle: ChartCandleUpdate },
    MarketData { market: PerpetualMarketData },
    MarketPrices { prices: HashMap<String, f64> },
    SubscriptionResponse { subscription_type: String },
    Unknown,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_encode_websocket_request() {
        let request = HyperliquidRequest {
            method: HyperliquidMethod::Subscribe,
            subscription: HyperliquidSubscription::Candle {
                symbol: "ETH".to_string(),
                interval: "30m".to_string(),
            },
        };

        assert_eq!(
            serde_json::to_value(request).unwrap(),
            json!({
                "method": "subscribe",
                "subscription": {
                    "type": "candle",
                    "coin": "ETH",
                    "interval": "30m",
                },
            })
        );
    }
}

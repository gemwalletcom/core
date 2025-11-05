use crate::core::actions::SLIPPAGE_BUFFER_PERCENT;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum TpslType {
    #[serde(rename = "tp")]
    TakeProfit,
    #[serde(rename = "sl")]
    StopLoss,
}

// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.
#[derive(Clone, Serialize, Deserialize)]
pub struct PlaceOrder {
    pub r#type: String,
    pub orders: Vec<Order>,
    pub grouping: Grouping,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder: Option<Builder>,
}

impl PlaceOrder {
    pub fn new(orders: Vec<Order>, grouping: Grouping, builder: Option<Builder>) -> Self {
        Self {
            r#type: "order".to_string(),
            orders,
            grouping,
            builder,
        }
    }
}

// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.
#[derive(Clone, Serialize, Deserialize)]
pub struct Order {
    #[serde(rename = "a")]
    pub asset: u32,
    #[serde(rename = "b")]
    pub is_buy: bool,
    #[serde(rename = "p")]
    pub price: String,
    /// Use "0" to apply to entire position (for position TP/SL orders)
    #[serde(rename = "s")]
    pub size: String,
    #[serde(rename = "r")]
    pub reduce_only: bool,
    #[serde(rename = "t")]
    pub order_type: OrderType,
    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrderType {
    Limit { limit: LimitOrder },
    Trigger { trigger: Trigger },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LimitOrder {
    pub tif: TimeInForce,
}

impl LimitOrder {
    pub fn new(tif: TimeInForce) -> Self {
        Self { tif }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Trigger {
    #[serde(rename = "isMarket")]
    pub is_market: bool,
    #[serde(rename = "triggerPx")]
    pub trigger_px: String,
    pub tpsl: TpslType,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum TimeInForce {
    #[serde(rename = "Alo")]
    AddLiquidityOnly,
    #[serde(rename = "Ioc")]
    ImmediateOrCancel,
    #[serde(rename = "Gtc")]
    GoodTillCancel,
    #[serde(rename = "FrontendMarket")]
    FrontendMarket,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Grouping {
    Na,
    NormalTpsl,
    PositionTpsl,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Builder {
    #[serde(rename = "b")]
    pub builder_address: String,
    #[serde(rename = "f")]
    pub fee: u32, // tenths of a basis point , 10 means 1bp
}

// https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/tick-and-lot-size
fn calculate_execution_price(trigger_px: &str, add_slippage: bool) -> String {
    let trigger: f64 = trigger_px.parse().unwrap_or(0.0);
    let execution_price = if add_slippage {
        trigger * (1.0 + SLIPPAGE_BUFFER_PERCENT)
    } else {
        trigger * (1.0 - SLIPPAGE_BUFFER_PERCENT)
    };

    // Round to 5 significant figures (max allowed by Hyperliquid)
    if execution_price != 0.0 && execution_price.is_finite() {
        let magnitude = execution_price.abs().log10().floor();
        let scale = 10_f64.powf(4.0 - magnitude);
        let rounded = (execution_price * scale).round() / scale;
        format!("{rounded:.6}").trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        format!("{execution_price:.6}").trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub fn make_market_order_type() -> OrderType {
    OrderType::Limit {
        limit: LimitOrder::new(TimeInForce::FrontendMarket),
    }
}

pub fn make_market_trigger_order_type(trigger_px: String, tpsl: TpslType) -> OrderType {
    OrderType::Trigger {
        trigger: Trigger {
            is_market: true,
            trigger_px,
            tpsl,
        },
    }
}

pub fn make_trigger_order(asset: u32, is_buy: bool, price: &str, size: &str, reduce_only: bool, trigger_px: String, tpsl: TpslType) -> Order {
    Order {
        asset,
        is_buy,
        price: price.to_string(),
        size: size.to_string(),
        reduce_only,
        order_type: make_market_trigger_order_type(trigger_px, tpsl),
        client_order_id: None,
    }
}

pub fn make_market_order(asset: u32, is_buy: bool, price: &str, size: &str, reduce_only: bool, builder: Option<Builder>) -> PlaceOrder {
    PlaceOrder::new(
        vec![Order {
            asset,
            is_buy,
            price: price.to_string(),
            size: size.to_string(),
            reduce_only,
            order_type: make_market_order_type(),
            client_order_id: None,
        }],
        Grouping::Na,
        builder,
    )
}

pub fn make_market_with_tp_sl(
    asset: u32,
    is_buy: bool,
    price: &str,
    size: &str,
    reduce_only: bool,
    tp_trigger: Option<String>,
    sl_trigger: Option<String>,
    builder: Option<Builder>,
) -> PlaceOrder {
    let mut orders = vec![Order {
        asset,
        is_buy,
        price: price.to_string(),
        size: size.to_string(),
        reduce_only,
        order_type: make_market_order_type(),
        client_order_id: None,
    }];

    if let Some(sl_trigger_px) = sl_trigger {
        let sl_execution_price = calculate_execution_price(&sl_trigger_px, true); // Market orders: add slippage
        orders.push(make_trigger_order(
            asset,
            !is_buy,
            &sl_execution_price,
            size,
            true, // TP/SL orders are always reduce_only=true
            sl_trigger_px,
            TpslType::StopLoss,
        ));
    }

    if let Some(tp_trigger_px) = tp_trigger {
        let tp_execution_price = calculate_execution_price(&tp_trigger_px, true); // Market orders: add slippage
        orders.push(make_trigger_order(
            asset,
            !is_buy,
            &tp_execution_price,
            size,
            true, // TP/SL orders are always reduce_only=true
            tp_trigger_px,
            TpslType::TakeProfit,
        ));
    }

    PlaceOrder::new(orders, Grouping::NormalTpsl, builder)
}

pub fn make_position_tp_sl(
    asset: u32,
    is_buy: bool,
    size: &str,
    tp_trigger: Option<String>,
    sl_trigger: Option<String>,
    builder: Option<Builder>,
) -> PlaceOrder {
    let mut orders = Vec::new();

    if let Some(sl_trigger_px) = sl_trigger {
        let sl_execution_price = calculate_execution_price(&sl_trigger_px, false); // Position orders: subtract slippage
        orders.push(make_trigger_order(
            asset,
            !is_buy,
            &sl_execution_price,
            size,
            true,
            sl_trigger_px,
            TpslType::StopLoss,
        ));
    }

    if let Some(tp_trigger_px) = tp_trigger {
        let tp_execution_price = calculate_execution_price(&tp_trigger_px, false); // Position orders: subtract slippage
        orders.push(make_trigger_order(
            asset,
            !is_buy,
            &tp_execution_price,
            size,
            true,
            tp_trigger_px,
            TpslType::TakeProfit,
        ));
    }

    PlaceOrder::new(orders, Grouping::PositionTpsl, builder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_execution_price_rounds_to_5_sig_figs() {
        let result = calculate_execution_price("156.66", false);
        assert_eq!(result, "144.13");

        let result = calculate_execution_price("100", true);
        assert_eq!(result, "108");

        let result = calculate_execution_price("1234.56", false);
        assert_eq!(result, "1135.8");
    }

    #[test]
    fn test_calculate_execution_price_handles_small_values() {
        let result = calculate_execution_price("0.12345", true);
        let parsed: f64 = result.parse().unwrap();
        assert!(parsed > 0.0 && parsed < 1.0);
    }

    #[test]
    fn test_calculate_execution_price_handles_zero() {
        let result = calculate_execution_price("0", false);
        assert_eq!(result, "0");
    }

    #[test]
    fn test_calculate_execution_price_trims_trailing_zeros() {
        let result = calculate_execution_price("100", false);
        assert!(!result.ends_with(".0"));
    }
}

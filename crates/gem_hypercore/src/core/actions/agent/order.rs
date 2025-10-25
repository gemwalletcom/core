use crate::core::actions::SLIPPAGE_BUFFER_PERCENT;

// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.
#[derive(Clone, serde::Serialize)]
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
#[derive(Clone, serde::Serialize)]
pub struct Order {
    #[serde(rename = "a")]
    pub asset: u32,
    #[serde(rename = "b")]
    pub is_buy: bool,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "s")]
    pub size: String,
    #[serde(rename = "r")]
    pub reduce_only: bool,
    #[serde(rename = "t")]
    pub order_type: OrderType,
    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

#[derive(Clone, serde::Serialize)]
#[serde(untagged)]
pub enum OrderType {
    Limit { limit: LimitOrder },
    Trigger { trigger: Trigger },
}

#[derive(Clone, serde::Serialize)]
pub struct LimitOrder {
    pub tif: TimeInForce,
}

impl LimitOrder {
    pub fn new(tif: TimeInForce) -> Self {
        Self { tif }
    }
}

#[derive(Clone, serde::Serialize)]
pub struct Trigger {
    #[serde(rename = "isMarket")]
    pub is_market: bool,
    #[serde(rename = "triggerPx")]
    pub trigger_px: String,
    pub tpsl: TpslType,
}

#[derive(Clone, serde::Serialize, Debug, PartialEq)]
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

#[derive(Clone, serde::Serialize)]
pub enum TpslType {
    #[serde(rename = "tp")]
    TakeProfit,
    #[serde(rename = "sl")]
    StopLoss,
}

#[derive(Clone, serde::Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Grouping {
    Na,
    NormalTpsl,
    PositionTpsl,
}

#[derive(Clone, serde::Serialize)]
pub struct Builder {
    #[serde(rename = "b")]
    pub builder_address: String,
    #[serde(rename = "f")]
    pub fee: u32, // tenths of a basis point , 10 means 1bp
}

fn calculate_execution_price(trigger_px: &str, add_slippage: bool) -> String {
    let trigger: f64 = trigger_px.parse().unwrap_or(0.0);
    let execution_price = if add_slippage {
        trigger * (1.0 + SLIPPAGE_BUFFER_PERCENT)
    } else {
        trigger * (1.0 - SLIPPAGE_BUFFER_PERCENT)
    };
    format!("{execution_price:.6}").trim_end_matches('0').trim_end_matches('.').to_string()
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
            size, // Note: "0" means apply to entire position
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
            size, // Note: "0" means apply to entire position
            true,
            tp_trigger_px,
            TpslType::TakeProfit,
        ));
    }

    PlaceOrder::new(orders, Grouping::PositionTpsl, builder)
}

// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// This must match the exact order from Python SDK
// Do not change field order unless you know the exact Python order.
#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperPlaceOrder {
    pub r#type: String,
    pub orders: Vec<HyperOrder>,
    pub grouping: HyperGrouping,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder: Option<HyperBuilder>,
}

impl HyperPlaceOrder {
    pub fn new(orders: Vec<HyperOrder>, grouping: HyperGrouping, builder: Option<HyperBuilder>) -> Self {
        Self {
            r#type: "order".to_string(),
            orders,
            grouping,
            builder,
        }
    }
}

// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// This must match the exact order from Python SDK
// Do not change field order unless you know the exact Python order.
#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperOrder {
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
    pub order_type: HyperOrderType,
    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

#[derive(uniffi::Enum, serde::Serialize)]
#[serde(untagged)]
pub enum HyperOrderType {
    Limit { limit: HyperLimitOrder },
    Trigger { trigger: HyperTrigger },
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperLimitOrder {
    pub tif: HyperTimeInForce,
}

impl HyperLimitOrder {
    pub fn new(tif: HyperTimeInForce) -> Self {
        Self { tif }
    }
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperTrigger {
    #[serde(rename = "isMarket")]
    pub is_market: bool,
    #[serde(rename = "triggerPx")]
    pub trigger_px: String,
    pub tpsl: HyperTpslType,
}

#[derive(uniffi::Enum, serde::Serialize, Debug, PartialEq)]
pub enum HyperTimeInForce {
    #[serde(rename = "Alo")]
    AddLiquidityOnly,
    #[serde(rename = "Ioc")]
    ImmediateOrCancel,
    #[serde(rename = "Gtc")]
    GoodTillCancel,
    #[serde(rename = "FrontendMarket")]
    FrontendMarket,
}

#[derive(uniffi::Enum, serde::Serialize)]
pub enum HyperTpslType {
    #[serde(rename = "tp")]
    TakeProfit,
    #[serde(rename = "sl")]
    StopLoss,
}

#[derive(uniffi::Enum, serde::Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum HyperGrouping {
    Na,
    NormalTpsl,
    PositionTpsl,
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperBuilder {
    #[serde(rename = "b")]
    pub builder_address: String,
    #[serde(rename = "f")]
    pub fee: u32, // tenths of a basis point , 10 means 1bp
}

pub fn make_market_close(asset: u32, price: String, size: String, reduce_only: bool) -> HyperPlaceOrder {
    HyperPlaceOrder::new(
        vec![HyperOrder {
            asset,
            is_buy: false,
            price,
            reduce_only,
            size,
            order_type: HyperOrderType::Limit {
                limit: HyperLimitOrder::new(HyperTimeInForce::FrontendMarket),
            },
            client_order_id: None,
        }],
        HyperGrouping::Na,
        None,
    )
}

pub fn make_market_open(asset: u32, is_buy: bool, price: String, size: String, reduce_only: bool) -> HyperPlaceOrder {
    HyperPlaceOrder::new(
        vec![HyperOrder {
            asset,
            is_buy,
            price,
            reduce_only,
            size,
            order_type: HyperOrderType::Limit {
                limit: HyperLimitOrder::new(HyperTimeInForce::FrontendMarket),
            },
            client_order_id: None,
        }],
        HyperGrouping::Na,
        None,
    )
}

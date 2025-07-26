#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperPlaceOrder {
    #[serde(rename = "type")]
    pub action_type: String,
    pub orders: Vec<HyperOrder>,
    pub grouping: HyperGrouping,
    pub builder: Option<HyperBuilder>,
}

impl HyperPlaceOrder {
    pub fn new(orders: Vec<HyperOrder>, grouping: HyperGrouping, builder: Option<HyperBuilder>) -> Self {
        Self {
            action_type: "order".to_string(),
            orders,
            grouping,
            builder,
        }
    }
}

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
    #[serde(rename = "c")]
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

#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperTrigger {
    #[serde(rename = "isMarket")]
    pub is_market: bool,
    #[serde(rename = "triggerPx")]
    pub trigger_px: String,
    pub tpsl: HyperTpslType,
}

#[derive(uniffi::Enum, serde::Serialize)]
pub enum HyperTimeInForce {
    #[serde(rename = "Alo")]
    AddLiquidityOnly,
    #[serde(rename = "Ioc")]
    ImmediateOrCancel,
    #[serde(rename = "Gtc")]
    GoodTillCancel,
}

#[derive(uniffi::Enum, serde::Serialize)]
pub enum HyperTpslType {
    #[serde(rename = "tp")]
    TakeProfit,
    #[serde(rename = "sl")]
    StopLoss,
}

#[derive(uniffi::Enum, serde::Serialize)]
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

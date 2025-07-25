#[derive(uniffi::Record, serde::Serialize)]
pub struct PlaceOrder {
    // type: "order"
    pub orders: Vec<Order>,
    pub grouping: Grouping,
    pub builder: Option<Builder>,
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct Order {
    pub a: u32,            // asset
    pub b: bool,           // isBuy
    pub p: String,         // price
    pub s: String,         // size
    pub r: bool,           // reduceOnly
    pub t: OrderType,      // type
    pub c: Option<String>, // cloid (client order id)
}

#[derive(uniffi::Enum, serde::Serialize)]
#[serde(untagged)]
pub enum OrderType {
    Limit { limit: LimitOrder },
    Trigger { trigger: TriggerOrder },
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct LimitOrder {
    pub tif: TimeInForce,
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct TriggerOrder {
    #[serde(rename = "isMarket")]
    pub is_market: bool,
    #[serde(rename = "triggerPx")]
    pub trigger_px: String,
    pub tpsl: TpslType,
}

#[derive(uniffi::Enum, serde::Serialize)]
pub enum TimeInForce {
    Alo, // add liquidity only
    Ioc, // immediate or cancel
    Gtc, // good till cancel
}

#[derive(uniffi::Enum, serde::Serialize)]
pub enum TpslType {
    #[serde(rename = "tp")]
    TakeProfit,
    #[serde(rename = "sl")]
    StopLoss,
}

#[derive(uniffi::Enum, serde::Serialize)]
pub enum Grouping {
    #[serde(rename = "na")]
    Na,
    #[serde(rename = "normalTpsl")]
    NormalTpsl,
    #[serde(rename = "positionTpsl")]
    PositionTpsl,
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct Builder {
    pub b: String, // address
    pub f: u32,    // fee in tenths of basis point
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct CancelOrder {
    pub cancels: Vec<Cancel>,
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct Cancel {
    pub a: u32, // asset
    pub o: u32, // oid (order id)
}

#[derive(uniffi::Record, serde::Serialize)]
pub struct WithdrawalRequest {
    // hyperliquid_chain: Mainnet
    #[serde(rename = "signatureChainId")]
    pub signature_chain_id: String,
    pub amount: String,
    pub time: u64,
    pub destination: String,
}

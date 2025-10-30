use alloy_primitives::hex;
use gem_hypercore::core::actions;

// Order types
pub type HyperPlaceOrder = actions::PlaceOrder;
pub type HyperOrder = actions::Order;
pub type HyperOrderType = actions::OrderType;
pub type HyperLimitOrder = actions::LimitOrder;
pub type HyperTrigger = actions::Trigger;
pub type HyperTimeInForce = actions::TimeInForce;
pub type HyperTpslType = actions::TpslType;
pub type HyperGrouping = actions::Grouping;
pub type HyperBuilder = actions::Builder;

// Action types
pub type HyperApproveAgent = actions::ApproveAgent;
pub type HyperApproveBuilderFee = actions::ApproveBuilderFee;
pub type HyperCDeposit = actions::CDeposit;
pub type HyperCWithdraw = actions::CWithdraw;
pub type HyperCancel = actions::Cancel;
pub type HyperCancelOrder = actions::CancelOrder;
pub type HyperSetReferrer = actions::SetReferrer;
pub type HyperSpotSend = actions::SpotSend;
pub type HyperTokenDelegate = actions::TokenDelegate;
pub type HyperUpdateLeverage = actions::UpdateLeverage;
pub type HyperUsdClassTransfer = actions::UsdClassTransfer;
pub type HyperUsdSend = actions::UsdSend;
pub type HyperWithdrawalRequest = actions::WithdrawalRequest;

// Order related remote types
#[uniffi::remote(Record)]
pub struct HyperPlaceOrder {
    pub r#type: String,
    pub orders: Vec<HyperOrder>,
    pub grouping: HyperGrouping,
    pub builder: Option<HyperBuilder>,
}

#[uniffi::remote(Record)]
pub struct HyperOrder {
    pub asset: u32,
    pub is_buy: bool,
    pub price: String,
    pub size: String,
    pub reduce_only: bool,
    pub order_type: HyperOrderType,
    pub client_order_id: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum HyperOrderType {
    Limit { limit: HyperLimitOrder },
    Trigger { trigger: HyperTrigger },
}

#[uniffi::remote(Record)]
pub struct HyperLimitOrder {
    pub tif: HyperTimeInForce,
}

#[uniffi::remote(Record)]
pub struct HyperTrigger {
    pub is_market: bool,
    pub trigger_px: String,
    pub tpsl: HyperTpslType,
}

#[uniffi::remote(Enum)]
pub enum HyperTimeInForce {
    AddLiquidityOnly,
    ImmediateOrCancel,
    GoodTillCancel,
    FrontendMarket,
}

#[uniffi::remote(Enum)]
pub enum HyperTpslType {
    TakeProfit,
    StopLoss,
}

#[uniffi::remote(Enum)]
pub enum HyperGrouping {
    Na,
    NormalTpsl,
    PositionTpsl,
}

#[uniffi::remote(Record)]
pub struct HyperBuilder {
    pub builder_address: String,
    pub fee: u32,
}

// Action type remote declarations
#[uniffi::remote(Record)]
pub struct HyperApproveAgent {
    pub agent_address: String,
    pub agent_name: String,
    pub hyperliquid_chain: String,
    pub nonce: u64,
    pub signature_chain_id: String,
    pub r#type: String,
}

#[uniffi::remote(Record)]
pub struct HyperApproveBuilderFee {
    pub max_fee_rate: String,
    pub builder: String,
    pub hyperliquid_chain: String,
    pub nonce: u64,
    pub signature_chain_id: String,
    pub r#type: String,
}

#[uniffi::remote(Record)]
pub struct HyperCDeposit {
    pub wei: u64,
    pub nonce: u64,
    pub hyperliquid_chain: String,
    pub signature_chain_id: String,
    pub r#type: String,
}

#[uniffi::remote(Record)]
pub struct HyperCWithdraw {
    pub wei: u64,
    pub nonce: u64,
    pub hyperliquid_chain: String,
    pub signature_chain_id: String,
    pub r#type: String,
}

#[uniffi::remote(Record)]
pub struct HyperCancel {
    pub r#type: String,
    pub cancels: Vec<HyperCancelOrder>,
}

#[uniffi::remote(Record)]
pub struct HyperCancelOrder {
    pub asset: u32,
    pub order_id: u64,
}

#[uniffi::remote(Record)]
pub struct HyperSetReferrer {
    pub r#type: String,
    pub code: String,
}

#[uniffi::remote(Record)]
pub struct HyperSpotSend {
    pub destination: String,
    pub token: String,
    pub amount: String,
    pub time: u64,
    pub hyperliquid_chain: String,
    pub signature_chain_id: String,
    pub r#type: String,
}

#[uniffi::remote(Record)]
pub struct HyperTokenDelegate {
    pub validator: String,
    pub wei: u64,
    pub is_undelegate: bool,
    pub nonce: u64,
    pub hyperliquid_chain: String,
    pub signature_chain_id: String,
    pub r#type: String,
}

#[uniffi::remote(Record)]
pub struct HyperUpdateLeverage {
    pub r#type: String,
    pub asset: u32,
    pub is_cross: bool,
    pub leverage: u64,
}

#[uniffi::remote(Record)]
pub struct HyperUsdClassTransfer {
    pub r#type: String,
    pub amount: String,
    pub to_perp: bool,
    pub nonce: u64,
    pub hyperliquid_chain: String,
    pub signature_chain_id: String,
}

#[uniffi::remote(Record)]
pub struct HyperUsdSend {
    pub destination: String,
    pub amount: String,
    pub time: u64,
    pub hyperliquid_chain: String,
    pub signature_chain_id: String,
    pub r#type: String,
}

#[uniffi::remote(Record)]
pub struct HyperWithdrawalRequest {
    pub amount: String,
    pub destination: String,
    pub hyperliquid_chain: String,
    pub signature_chain_id: String,
    pub time: u64,
    pub r#type: String,
}

// Model factory functions

pub fn hyper_make_approve_agent(name: String, address: String, nonce: u64) -> HyperApproveAgent {
    actions::ApproveAgent::new(address, name, nonce)
}

pub fn hyper_make_approve_builder(max_fee_rate: String, builder: String, nonce: u64) -> HyperApproveBuilderFee {
    actions::ApproveBuilderFee::new(max_fee_rate, builder, nonce)
}

pub fn hyper_make_market_order(asset: u32, is_buy: bool, price: String, size: String, reduce_only: bool, builder: Option<HyperBuilder>) -> HyperPlaceOrder {
    actions::make_market_order(asset, is_buy, &price, &size, reduce_only, builder)
}

pub fn hyper_make_market_with_tp_sl(
    asset: u32,
    is_buy: bool,
    price: String,
    size: String,
    reduce_only: bool,
    tp_trigger: Option<String>,
    sl_trigger: Option<String>,
    builder: Option<HyperBuilder>,
) -> HyperPlaceOrder {
    actions::make_market_with_tp_sl(asset, is_buy, &price, &size, reduce_only, tp_trigger, sl_trigger, builder)
}

pub fn hyper_build_signed_request(signature: String, action: String, timestamp: u64) -> String {
    let sig_bytes = hex::decode(&signature).unwrap();

    let r = hex::encode_prefixed(&sig_bytes[0..32]);
    let s = hex::encode_prefixed(&sig_bytes[32..64]);
    let v = sig_bytes[64] as u64;

    let action_json: serde_json::Value = serde_json::from_str(&action).unwrap();

    let signed_request = serde_json::json!({
        "action": action_json,
        "signature": {
            "r": r,
            "s": s,
            "v": v
        },
        "nonce": timestamp,
        "isFrontend": true
    });

    serde_json::to_string(&signed_request).unwrap()
}

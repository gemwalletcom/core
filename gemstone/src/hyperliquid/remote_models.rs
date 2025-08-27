// Remote UniFFI models for Hyperliquid types from gem_hypercore
use alloy_primitives::hex;
use gem_hypercore::actions;
use gem_hypercore::core::hypercore;

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

// UniFFI functions that directly use gem_hypercore functionality
#[uniffi::export]
pub fn hyper_core_place_order_typed_data(order: HyperPlaceOrder, nonce: u64) -> String {
    hypercore::place_order_typed_data(order, nonce)
}

#[uniffi::export]
pub fn hyper_core_set_referrer_typed_data(referrer: HyperSetReferrer, nonce: u64) -> String {
    hypercore::set_referrer_typed_data(referrer, nonce)
}

#[uniffi::export]
pub fn hyper_core_update_leverage_typed_data(update_leverage: HyperUpdateLeverage, nonce: u64) -> String {
    hypercore::update_leverage_typed_data(update_leverage, nonce)
}

#[uniffi::export]
pub fn hyper_core_withdrawal_request_typed_data(request: HyperWithdrawalRequest) -> String {
    hypercore::withdrawal_request_typed_data(request)
}

#[uniffi::export]
pub fn hyper_core_approve_agent_typed_data(agent: HyperApproveAgent) -> String {
    hypercore::approve_agent_typed_data(agent)
}

#[uniffi::export]
pub fn hyper_core_approve_builder_fee_typed_data(fee: HyperApproveBuilderFee) -> String {
    hypercore::approve_builder_fee_typed_data(fee)
}

#[uniffi::export]
pub fn hyper_core_transfer_to_hyper_evm_typed_data(spot_send: HyperSpotSend) -> String {
    hypercore::transfer_to_hyper_evm_typed_data(spot_send)
}

#[uniffi::export]
pub fn hyper_core_send_spot_token_to_address_typed_data(spot_send: HyperSpotSend) -> String {
    hypercore::send_spot_token_to_address_typed_data(spot_send)
}

#[uniffi::export]
pub fn hyper_core_send_perps_usd_to_address_typed_data(usd_send: HyperUsdSend) -> String {
    hypercore::send_perps_usd_to_address_typed_data(usd_send)
}

#[uniffi::export]
pub fn hyper_core_transfer_spot_to_perps_typed_data(usd_class_transfer: HyperUsdClassTransfer) -> String {
    hypercore::transfer_spot_to_perps_typed_data(usd_class_transfer)
}

#[uniffi::export]
pub fn hyper_core_transfer_perps_to_spot_typed_data(usd_class_transfer: HyperUsdClassTransfer) -> String {
    hypercore::transfer_perps_to_spot_typed_data(usd_class_transfer)
}

#[uniffi::export]
pub fn hyper_core_c_deposit_typed_data(c_deposit: HyperCDeposit) -> String {
    hypercore::c_deposit_typed_data(c_deposit)
}

#[uniffi::export]
pub fn hyper_core_token_delegate_typed_data(token_delegate: HyperTokenDelegate) -> String {
    hypercore::token_delegate_typed_data(token_delegate)
}

// Model factory functions
#[uniffi::export]
pub fn hyper_make_approve_agent(name: String, address: String, nonce: u64) -> HyperApproveAgent {
    actions::ApproveAgent::new(address, name, nonce)
}

#[uniffi::export]
pub fn hyper_make_approve_builder(max_fee_rate: String, builder: String, nonce: u64) -> HyperApproveBuilderFee {
    actions::ApproveBuilderFee::new(max_fee_rate, builder, nonce)
}

#[uniffi::export]
pub fn hyper_make_market_order(asset: u32, is_buy: bool, price: String, size: String, reduce_only: bool, builder: Option<HyperBuilder>) -> HyperPlaceOrder {
    actions::make_market_order(asset, is_buy, &price, &size, reduce_only, builder)
}

#[uniffi::export]
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

#[uniffi::export]
pub fn hyper_serialize_order(order: &HyperPlaceOrder) -> String {
    serde_json::to_string(order).unwrap()
}

#[uniffi::export]
pub fn hyper_make_cancel_orders(orders: Vec<HyperCancelOrder>) -> HyperCancel {
    actions::Cancel::new(orders)
}

#[uniffi::export]
pub fn hyper_serialize_cancel_action(cancel_action: &HyperCancel) -> String {
    serde_json::to_string(cancel_action).unwrap()
}

#[uniffi::export]
pub fn hyper_make_position_tp_sl(
    asset: u32,
    is_buy: bool,
    size: String,
    tp_trigger: String,
    sl_trigger: String,
    builder: Option<HyperBuilder>,
) -> HyperPlaceOrder {
    actions::make_position_tp_sl(asset, is_buy, &size, Some(tp_trigger), Some(sl_trigger), builder)
}

#[uniffi::export]
pub fn hyper_make_withdraw(amount: String, address: String, time: u64) -> HyperWithdrawalRequest {
    actions::WithdrawalRequest::new(amount, time, address)
}

#[uniffi::export]
pub fn hyper_make_set_referrer(referrer: String) -> HyperSetReferrer {
    actions::SetReferrer::new(referrer)
}

#[uniffi::export]
pub fn hyper_serialize_set_referrer(set_referrer: &HyperSetReferrer) -> String {
    serde_json::to_string(set_referrer).unwrap()
}

#[uniffi::export]
pub fn hyper_make_update_leverage(asset: u32, is_cross: bool, leverage: u64) -> HyperUpdateLeverage {
    actions::UpdateLeverage::new(asset, is_cross, leverage)
}

#[uniffi::export]
pub fn hyper_serialize_update_leverage(update_leverage: &HyperUpdateLeverage) -> String {
    serde_json::to_string(update_leverage).unwrap()
}

#[uniffi::export]
pub fn hyper_transfer_to_hyper_evm(amount: String, time: u64, token: String) -> HyperSpotSend {
    actions::SpotSend::new(amount, actions::HYPERCORE_EVM_BRIDGE_ADDRESS.to_string(), time, token)
}

#[uniffi::export]
pub fn hyper_send_spot_token_to_address(amount: String, destination: String, time: u64, token: String) -> HyperSpotSend {
    actions::SpotSend::new(amount, destination, time, token)
}

#[uniffi::export]
pub fn hyper_serialize_spot_send(spot_send: &HyperSpotSend) -> String {
    serde_json::to_string(spot_send).unwrap()
}

#[uniffi::export]
pub fn hyper_send_perps_usd_to_address(amount: String, destination: String, time: u64) -> HyperUsdSend {
    actions::UsdSend::new(amount, destination, time)
}

#[uniffi::export]
pub fn hyper_serialize_usd_send(usd_send: &HyperUsdSend) -> String {
    serde_json::to_string(usd_send).unwrap()
}

#[uniffi::export]
pub fn hyper_transfer_spot_to_perps(amount: String, nonce: u64) -> HyperUsdClassTransfer {
    actions::UsdClassTransfer::new(amount, true, nonce)
}

#[uniffi::export]
pub fn hyper_transfer_perps_to_spot(amount: String, nonce: u64) -> HyperUsdClassTransfer {
    actions::UsdClassTransfer::new(amount, false, nonce)
}

#[uniffi::export]
pub fn hyper_serialize_usd_class_transfer(usd_class_transfer: &HyperUsdClassTransfer) -> String {
    serde_json::to_string(usd_class_transfer).unwrap()
}

#[uniffi::export]
pub fn hyper_make_transfer_to_staking(wei: u64, nonce: u64) -> HyperCDeposit {
    actions::CDeposit::new(wei, nonce)
}

#[uniffi::export]
pub fn hyper_serialize_c_deposit(c_deposit: &HyperCDeposit) -> String {
    serde_json::to_string(c_deposit).unwrap()
}

#[uniffi::export]
pub fn hyper_make_delegate(validator: String, wei: u64, nonce: u64) -> HyperTokenDelegate {
    actions::TokenDelegate::new(validator, wei, false, nonce)
}

#[uniffi::export]
pub fn hyper_make_undelegate(validator: String, wei: u64, nonce: u64) -> HyperTokenDelegate {
    actions::TokenDelegate::new(validator, wei, true, nonce)
}

#[uniffi::export]
pub fn hyper_serialize_token_delegate(token_delegate: &HyperTokenDelegate) -> String {
    serde_json::to_string(token_delegate).unwrap()
}

#[uniffi::export]
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

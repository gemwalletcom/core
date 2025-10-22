pub mod remote_models;

// Re-export the types from remote_models
pub use remote_models::*;

use crate::{GemstoneError, alien::AlienSigner};
use serde::Serialize;
use serde_json::{self, Value};
use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct HyperCoreModelFactory;

#[uniffi::export]
impl HyperCoreModelFactory {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    // Order factory methods
    pub fn make_market_order(
        &self,
        asset: u32,
        is_buy: bool,
        price: String,
        size: String,
        reduce_only: bool,
        builder: Option<HyperBuilder>,
    ) -> HyperPlaceOrder {
        hyper_make_market_order(asset, is_buy, price, size, reduce_only, builder)
    }

    pub fn make_market_with_tp_sl(
        &self,
        asset: u32,
        is_buy: bool,
        price: String,
        size: String,
        reduce_only: bool,
        tp_trigger: Option<String>,
        sl_trigger: Option<String>,
        builder: Option<HyperBuilder>,
    ) -> HyperPlaceOrder {
        hyper_make_market_with_tp_sl(asset, is_buy, price, size, reduce_only, tp_trigger, sl_trigger, builder)
    }

    pub fn make_position_tp_sl(
        &self,
        asset: u32,
        is_buy: bool,
        size: String,
        tp_trigger: String,
        sl_trigger: String,
        builder: Option<HyperBuilder>,
    ) -> HyperPlaceOrder {
        hyper_make_position_tp_sl(asset, is_buy, size, tp_trigger, sl_trigger, builder)
    }

    pub fn serialize_order(&self, order: &HyperPlaceOrder) -> String {
        serde_json::to_string(order).unwrap()
    }

    // Cancel order methods
    pub fn make_cancel_orders(&self, orders: Vec<HyperCancelOrder>) -> HyperCancel {
        hyper_make_cancel_orders(orders)
    }

    pub fn serialize_cancel_action(&self, cancel_action: &HyperCancel) -> String {
        serde_json::to_string(cancel_action).unwrap()
    }

    // Account management methods
    pub fn make_set_referrer(&self, referrer: String) -> HyperSetReferrer {
        hyper_make_set_referrer(referrer)
    }

    pub fn serialize_set_referrer(&self, set_referrer: &HyperSetReferrer) -> String {
        serde_json::to_string(set_referrer).unwrap()
    }

    pub fn make_update_leverage(&self, asset: u32, is_cross: bool, leverage: u64) -> HyperUpdateLeverage {
        hyper_make_update_leverage(asset, is_cross, leverage)
    }

    pub fn serialize_update_leverage(&self, update_leverage: &HyperUpdateLeverage) -> String {
        serde_json::to_string(update_leverage).unwrap()
    }

    // Withdrawal methods
    pub fn make_withdraw(&self, amount: String, address: String, nonce: u64) -> HyperWithdrawalRequest {
        hyper_make_withdraw(amount, address, nonce)
    }

    // Spot transfer methods
    pub fn transfer_to_hyper_evm(&self, amount: String, time: u64, token: String) -> HyperSpotSend {
        hyper_transfer_to_hyper_evm(amount, time, token)
    }

    pub fn send_spot_token_to_address(&self, amount: String, destination: String, time: u64, token: String) -> HyperSpotSend {
        hyper_send_spot_token_to_address(amount, destination, time, token)
    }

    pub fn serialize_spot_send(&self, spot_send: &HyperSpotSend) -> String {
        serde_json::to_string(spot_send).unwrap()
    }

    // USD transfer methods
    pub fn send_perps_usd_to_address(&self, amount: String, destination: String, time: u64) -> HyperUsdSend {
        hyper_send_perps_usd_to_address(amount, destination, time)
    }

    pub fn serialize_usd_send(&self, usd_send: &HyperUsdSend) -> String {
        serde_json::to_string(usd_send).unwrap()
    }

    pub fn transfer_spot_to_perps(&self, amount: String, nonce: u64) -> HyperUsdClassTransfer {
        hyper_transfer_spot_to_perps(amount, nonce)
    }

    pub fn transfer_perps_to_spot(&self, amount: String, nonce: u64) -> HyperUsdClassTransfer {
        hyper_transfer_perps_to_spot(amount, nonce)
    }

    pub fn serialize_usd_class_transfer(&self, usd_class_transfer: &HyperUsdClassTransfer) -> String {
        serde_json::to_string(usd_class_transfer).unwrap()
    }

    // Staking methods
    pub fn make_transfer_to_staking(&self, wei: u64, nonce: u64) -> HyperCDeposit {
        hyper_make_transfer_to_staking(wei, nonce)
    }

    pub fn make_withdraw_from_staking(&self, wei: u64, nonce: u64) -> HyperCWithdraw {
        hyper_make_withdraw_from_staking(wei, nonce)
    }

    pub fn serialize_c_deposit(&self, c_deposit: &HyperCDeposit) -> String {
        serde_json::to_string(c_deposit).unwrap()
    }

    pub fn serialize_c_withdraw(&self, c_withdraw: &HyperCWithdraw) -> String {
        serde_json::to_string(c_withdraw).unwrap()
    }

    pub fn make_delegate(&self, validator: String, wei: u64, nonce: u64) -> HyperTokenDelegate {
        hyper_make_delegate(validator, wei, nonce)
    }

    pub fn make_undelegate(&self, validator: String, wei: u64, nonce: u64) -> HyperTokenDelegate {
        hyper_make_undelegate(validator, wei, nonce)
    }

    pub fn serialize_token_delegate(&self, token_delegate: &HyperTokenDelegate) -> String {
        serde_json::to_string(token_delegate).unwrap()
    }

    // Approval methods
    pub fn make_approve_agent(&self, name: String, address: String, nonce: u64) -> HyperApproveAgent {
        hyper_make_approve_agent(name, address, nonce)
    }

    pub fn make_approve_builder(&self, max_fee_rate: String, builder: String, nonce: u64) -> HyperApproveBuilderFee {
        hyper_make_approve_builder(max_fee_rate, builder, nonce)
    }

    // Request building
    pub fn build_signed_request(&self, signature: String, action: String, timestamp: u64) -> String {
        hyper_build_signed_request(signature, action, timestamp)
    }
}

impl Default for HyperCoreModelFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(uniffi::Object)]
pub struct HyperCore {
    signer: Arc<dyn AlienSigner>,
}

#[uniffi::export]
impl HyperCore {
    #[uniffi::constructor]
    pub fn new(signer: Arc<dyn AlienSigner>) -> Self {
        Self { signer }
    }

    fn sign_action(&self, typed_data: String, action: String, timestamp: u64, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let signature = self.signer.sign_eip712(typed_data, private_key)?;
        Ok(hyper_build_signed_request(signature, action, timestamp))
    }

    pub fn sign_typed_action(&self, typed_data_json: String, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let typed_data: Value =
            serde_json::from_str(&typed_data_json).map_err(|err| GemstoneError::from(format!("Invalid typed data JSON: {err}")))?;

        let message = typed_data
            .get("message")
            .ok_or_else(|| GemstoneError::from("Typed data missing message field"))?;

        let timestamp = extract_timestamp(message)?;
        let action = serde_json::to_string(message)
            .map_err(|err| GemstoneError::from(format!("Failed to serialize action payload: {err}")))?;

        self.sign_action(typed_data_json, action, timestamp, private_key)
    }

    pub fn sign_spot_send(&self, spot_send: HyperSpotSend, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let timestamp = spot_send.time;
        sign_serialized_action(
            self,
            spot_send,
            timestamp,
            private_key,
            hyper_core_send_spot_token_to_address_typed_data,
            "spot send",
        )
    }

    pub fn sign_withdrawal_request(&self, request: HyperWithdrawalRequest, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let timestamp = request.time;
        sign_serialized_action(
            self,
            request,
            timestamp,
            private_key,
            hyper_core_withdrawal_request_typed_data,
            "withdrawal request",
        )
    }

    pub fn sign_approve_agent(&self, agent: HyperApproveAgent, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let timestamp = agent.nonce;
        sign_serialized_action(self, agent, timestamp, private_key, hyper_core_approve_agent_typed_data, "approve agent")
    }

    pub fn sign_approve_builder_fee(&self, fee: HyperApproveBuilderFee, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let timestamp = fee.nonce;
        sign_serialized_action(
            self,
            fee,
            timestamp,
            private_key,
            hyper_core_approve_builder_fee_typed_data,
            "approve builder fee",
        )
    }

    pub fn sign_set_referrer(&self, referrer: HyperSetReferrer, nonce: u64, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        sign_serialized_action(
            self,
            referrer,
            nonce,
            private_key,
            |value| hyper_core_set_referrer_typed_data(value, nonce),
            "set referrer",
        )
    }

    pub fn sign_c_deposit(&self, deposit: HyperCDeposit, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let timestamp = deposit.nonce;
        sign_serialized_action(self, deposit, timestamp, private_key, hyper_core_c_deposit_typed_data, "c deposit")
    }

    pub fn sign_c_withdraw(&self, withdraw: HyperCWithdraw, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let timestamp = withdraw.nonce;
        sign_serialized_action(self, withdraw, timestamp, private_key, hyper_core_c_withdraw_typed_data, "c withdraw")
    }

    pub fn sign_token_delegate(&self, delegate: HyperTokenDelegate, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let timestamp = delegate.nonce;
        sign_serialized_action(self, delegate, timestamp, private_key, hyper_core_token_delegate_typed_data, "token delegate")
    }

    pub fn sign_place_order(&self, order: HyperPlaceOrder, nonce: u64, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        sign_serialized_action(
            self,
            order,
            nonce,
            private_key,
            |value| hyper_core_place_order_typed_data(value, nonce),
            "place order",
        )
    }

    // EIP-712 typed data generation methods
    pub fn place_order_typed_data(&self, order: HyperPlaceOrder, nonce: u64) -> String {
        hyper_core_place_order_typed_data(order, nonce)
    }

    pub fn set_referrer_typed_data(&self, referrer: HyperSetReferrer, nonce: u64) -> String {
        hyper_core_set_referrer_typed_data(referrer, nonce)
    }

    pub fn update_leverage_typed_data(&self, update_leverage: HyperUpdateLeverage, nonce: u64) -> String {
        hyper_core_update_leverage_typed_data(update_leverage, nonce)
    }

    pub fn withdrawal_request_typed_data(&self, request: HyperWithdrawalRequest) -> String {
        hyper_core_withdrawal_request_typed_data(request)
    }

    pub fn approve_agent_typed_data(&self, agent: HyperApproveAgent) -> String {
        hyper_core_approve_agent_typed_data(agent)
    }

    pub fn approve_builder_fee_typed_data(&self, fee: HyperApproveBuilderFee) -> String {
        hyper_core_approve_builder_fee_typed_data(fee)
    }

    pub fn transfer_to_hyper_evm_typed_data(&self, spot_send: HyperSpotSend) -> String {
        hyper_core_transfer_to_hyper_evm_typed_data(spot_send)
    }

    pub fn send_spot_token_to_address_typed_data(&self, spot_send: HyperSpotSend) -> String {
        hyper_core_send_spot_token_to_address_typed_data(spot_send)
    }

    pub fn send_perps_usd_to_address_typed_data(&self, usd_send: HyperUsdSend) -> String {
        hyper_core_send_perps_usd_to_address_typed_data(usd_send)
    }

    pub fn transfer_spot_to_perps_typed_data(&self, usd_class_transfer: HyperUsdClassTransfer) -> String {
        hyper_core_transfer_spot_to_perps_typed_data(usd_class_transfer)
    }

    pub fn transfer_perps_to_spot_typed_data(&self, usd_class_transfer: HyperUsdClassTransfer) -> String {
        hyper_core_transfer_perps_to_spot_typed_data(usd_class_transfer)
    }

    pub fn c_deposit_typed_data(&self, c_deposit: HyperCDeposit) -> String {
        hyper_core_c_deposit_typed_data(c_deposit)
    }

    pub fn c_withdraw_typed_data(&self, c_withdraw: HyperCWithdraw) -> String {
        hyper_core_c_withdraw_typed_data(c_withdraw)
    }

    pub fn token_delegate_typed_data(&self, token_delegate: HyperTokenDelegate) -> String {
        hyper_core_token_delegate_typed_data(token_delegate)
    }
}

fn sign_serialized_action<T, F>(
    core: &HyperCore,
    value: T,
    timestamp: u64,
    private_key: Vec<u8>,
    typed_data_fn: F,
    action_name: &'static str,
) -> Result<String, GemstoneError>
where
    T: Serialize,
    F: FnOnce(T) -> String,
{
    let action = serde_json::to_string(&value)
        .map_err(|err| GemstoneError::from(format!("Failed to serialize {action_name} action: {err}")))?;
    let typed_data = typed_data_fn(value);
    core.sign_action(typed_data, action, timestamp, private_key)
}

fn extract_timestamp(message: &Value) -> Result<u64, GemstoneError> {
    if let Some(time) = message.get("time") {
        parse_numeric_field(time, "time")
    } else if let Some(nonce) = message.get("nonce") {
        parse_numeric_field(nonce, "nonce")
    } else {
        Err(GemstoneError::from("Typed data message missing time or nonce field"))
    }
}

fn parse_numeric_field(value: &Value, field: &str) -> Result<u64, GemstoneError> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| GemstoneError::from(format!("Typed data message.{field} is not a positive integer"))),
        Value::String(s) => s
            .parse::<u64>()
            .map_err(|err| GemstoneError::from(format!("Typed data message.{field} is not a valid u64: {err}"))),
        _ => Err(GemstoneError::from(format!(
            "Typed data message.{field} must be a string or number"
        ))),
    }
}

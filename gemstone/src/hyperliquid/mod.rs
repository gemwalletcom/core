pub mod remote_models;
pub use remote_models::*;

use crate::GemstoneError;
use gem_hypercore::{
    core::hypercore::{
        approve_agent_typed_data, approve_builder_fee_typed_data, c_deposit_typed_data, c_withdraw_typed_data, place_order_typed_data,
        send_spot_token_to_address_typed_data, set_referrer_typed_data, token_delegate_typed_data, withdrawal_request_typed_data,
    },
    models::timestamp::TimestampField,
};
use serde::Serialize;
use serde_json::{self, Value};
use signer::Signer;

fn sign_action(typed_data: String, action: String, timestamp: u64, private_key: &[u8]) -> Result<String, GemstoneError> {
    let signature = Signer::sign_eip712(&typed_data, private_key)?;
    Ok(hyper_build_signed_request(signature, action, timestamp))
}

fn sign_serialized_action<T, F>(value: T, timestamp: u64, private_key: &[u8], typed_data_fn: F, action_name: &'static str) -> Result<String, GemstoneError>
where
    T: Serialize,
    F: FnOnce(T) -> String,
{
    let action = serde_json::to_string(&value).map_err(|err| GemstoneError::from(format!("Failed to serialize {action_name} action: {err}")))?;
    let typed_data = typed_data_fn(value);
    sign_action(typed_data, action, timestamp, private_key)
}

pub fn hyper_core_sign_typed_action(typed_data_json: String, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let typed_data: Value = serde_json::from_str(&typed_data_json).map_err(|err| GemstoneError::from(format!("Invalid typed data JSON: {err}")))?;

    let message = typed_data
        .get("message")
        .ok_or_else(|| GemstoneError::from("Typed data missing message field"))?;

    let timestamp = serde_json::from_value::<TimestampField>(message.clone())
        .map_err(|err| GemstoneError::from(format!("Failed to parse time or nonce: {err}")))?
        .value;
    let action = serde_json::to_string(message).map_err(|err| GemstoneError::from(format!("Failed to serialize action payload: {err}")))?;

    sign_action(typed_data_json, action, timestamp, private_key.as_slice())
}

pub fn hyper_core_sign_spot_send(spot_send: HyperSpotSend, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let timestamp = spot_send.time;
    sign_serialized_action(spot_send, timestamp, private_key.as_slice(), send_spot_token_to_address_typed_data, "spot send")
}

pub fn hyper_core_sign_withdrawal_request(request: HyperWithdrawalRequest, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let timestamp = request.time;
    sign_serialized_action(request, timestamp, private_key.as_slice(), withdrawal_request_typed_data, "withdrawal request")
}

pub fn hyper_core_sign_approve_agent(agent: HyperApproveAgent, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let timestamp = agent.nonce;
    sign_serialized_action(agent, timestamp, private_key.as_slice(), approve_agent_typed_data, "approve agent")
}

pub fn hyper_core_sign_approve_builder_fee(fee: HyperApproveBuilderFee, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let timestamp = fee.nonce;
    sign_serialized_action(fee, timestamp, private_key.as_slice(), approve_builder_fee_typed_data, "approve builder fee")
}

pub fn hyper_core_sign_set_referrer(referrer: HyperSetReferrer, nonce: u64, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    sign_serialized_action(
        referrer,
        nonce,
        private_key.as_slice(),
        |value| set_referrer_typed_data(value, nonce),
        "set referrer",
    )
}

pub fn hyper_core_sign_c_deposit(deposit: HyperCDeposit, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let timestamp = deposit.nonce;
    sign_serialized_action(deposit, timestamp, private_key.as_slice(), c_deposit_typed_data, "c deposit")
}

pub fn hyper_core_sign_c_withdraw(withdraw: HyperCWithdraw, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let timestamp = withdraw.nonce;
    sign_serialized_action(withdraw, timestamp, private_key.as_slice(), c_withdraw_typed_data, "c withdraw")
}

pub fn hyper_core_sign_token_delegate(delegate: HyperTokenDelegate, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let timestamp = delegate.nonce;
    sign_serialized_action(delegate, timestamp, private_key.as_slice(), token_delegate_typed_data, "token delegate")
}

pub fn hyper_core_sign_place_order(order: HyperPlaceOrder, nonce: u64, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    sign_serialized_action(
        order,
        nonce,
        private_key.as_slice(),
        |value| place_order_typed_data(value, nonce),
        "place order",
    )
}

pub mod remote_models;

// Re-export the types from remote_models
pub use remote_models::*;

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
        hyper_serialize_order(order)
    }

    // Cancel order methods
    pub fn make_cancel_orders(&self, orders: Vec<HyperCancelOrder>) -> HyperCancel {
        hyper_make_cancel_orders(orders)
    }

    pub fn serialize_cancel_action(&self, cancel_action: &HyperCancel) -> String {
        hyper_serialize_cancel_action(cancel_action)
    }

    // Account management methods
    pub fn make_set_referrer(&self, referrer: String) -> HyperSetReferrer {
        hyper_make_set_referrer(referrer)
    }

    pub fn serialize_set_referrer(&self, set_referrer: &HyperSetReferrer) -> String {
        hyper_serialize_set_referrer(set_referrer)
    }

    pub fn make_update_leverage(&self, asset: u32, is_cross: bool, leverage: u64) -> HyperUpdateLeverage {
        hyper_make_update_leverage(asset, is_cross, leverage)
    }

    pub fn serialize_update_leverage(&self, update_leverage: &HyperUpdateLeverage) -> String {
        hyper_serialize_update_leverage(update_leverage)
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
        hyper_serialize_spot_send(spot_send)
    }

    // USD transfer methods
    pub fn send_perps_usd_to_address(&self, amount: String, destination: String, time: u64) -> HyperUsdSend {
        hyper_send_perps_usd_to_address(amount, destination, time)
    }

    pub fn serialize_usd_send(&self, usd_send: &HyperUsdSend) -> String {
        hyper_serialize_usd_send(usd_send)
    }

    pub fn transfer_spot_to_perps(&self, amount: String, nonce: u64) -> HyperUsdClassTransfer {
        hyper_transfer_spot_to_perps(amount, nonce)
    }

    pub fn transfer_perps_to_spot(&self, amount: String, nonce: u64) -> HyperUsdClassTransfer {
        hyper_transfer_perps_to_spot(amount, nonce)
    }

    pub fn serialize_usd_class_transfer(&self, usd_class_transfer: &HyperUsdClassTransfer) -> String {
        hyper_serialize_usd_class_transfer(usd_class_transfer)
    }

    // Staking methods
    pub fn make_transfer_to_staking(&self, wei: u64, nonce: u64) -> HyperCDeposit {
        hyper_make_transfer_to_staking(wei, nonce)
    }

    pub fn serialize_c_deposit(&self, c_deposit: &HyperCDeposit) -> String {
        hyper_serialize_c_deposit(c_deposit)
    }

    pub fn make_delegate(&self, validator: String, wei: u64, nonce: u64) -> HyperTokenDelegate {
        hyper_make_delegate(validator, wei, nonce)
    }

    pub fn make_undelegate(&self, validator: String, wei: u64, nonce: u64) -> HyperTokenDelegate {
        hyper_make_undelegate(validator, wei, nonce)
    }

    pub fn serialize_token_delegate(&self, token_delegate: &HyperTokenDelegate) -> String {
        hyper_serialize_token_delegate(token_delegate)
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
pub struct HyperCore;

#[uniffi::export]
impl HyperCore {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
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

    pub fn token_delegate_typed_data(&self, token_delegate: HyperTokenDelegate) -> String {
        hyper_core_token_delegate_typed_data(token_delegate)
    }
}

impl Default for HyperCore {
    fn default() -> Self {
        Self::new()
    }
}

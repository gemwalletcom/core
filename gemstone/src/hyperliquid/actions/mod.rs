pub mod approve_agent;
pub mod approve_builder_fee;
pub mod c_deposit;
pub mod cancel_order;
pub mod order;
pub mod set_referrer;
pub mod spot_send;
pub mod token_delegate;
pub mod update_leverage;
pub mod usd_class_transfer;
pub mod usd_send;
pub mod withdrawal;

pub use approve_agent::*;
pub use approve_builder_fee::*;
pub use c_deposit::*;
pub use cancel_order::*;
pub use order::*;
pub use set_referrer::*;
pub use spot_send::*;
pub use token_delegate::*;
pub use update_leverage::*;
pub use usd_class_transfer::*;
pub use usd_send::*;
pub use withdrawal::*;

pub const MAINNET: &str = "Mainnet";
pub const SIGNATURE_CHAIN_ID: &str = "0xa4b1";
pub const HYPERCORE_SIGNATURE_CHAIN_ID: &str = "0x3e7";
pub const HYPERCORE_EVM_BRIDGE_ADDRESS: &str = "0x2222222222222222222222222222222222222222";
pub const SLIPPAGE_BUFFER_PERCENT: f64 = 0.08;

use alloy_primitives::hex;

#[derive(uniffi::Object)]
pub struct HyperCoreModelFactory {}

#[uniffi::export]
impl HyperCoreModelFactory {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    fn make_approve_agent(&self, name: String, address: String, nonce: u64) -> HyperApproveAgent {
        HyperApproveAgent::new(address, name, nonce)
    }

    fn make_approve_builder(&self, max_fee_rate: String, builder: String, nonce: u64) -> HyperApproveBuilderFee {
        HyperApproveBuilderFee::new(max_fee_rate, builder, nonce)
    }

    fn make_market_order(&self, asset: u32, is_buy: bool, price: String, size: String, reduce_only: bool, builder: Option<HyperBuilder>) -> HyperPlaceOrder {
        order::make_market_order(asset, is_buy, &price, &size, reduce_only, builder)
    }

    fn make_market_with_tp_sl(
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
        order::make_market_with_tp_sl(asset, is_buy, &price, &size, reduce_only, tp_trigger, sl_trigger, builder)
    }

    fn serialize_order(&self, order: &HyperPlaceOrder) -> String {
        serde_json::to_string(order).unwrap()
    }

    fn make_cancel_orders(&self, orders: Vec<HyperCancelOrder>) -> HyperCancel {
        HyperCancel::new(orders)
    }

    fn serialize_cancel_action(&self, cancel_action: &HyperCancel) -> String {
        serde_json::to_string(cancel_action).unwrap()
    }

    fn make_position_tp_sl(
        &self,
        asset: u32,
        is_buy: bool,
        size: String,
        tp_trigger: String,
        sl_trigger: String,
        builder: Option<HyperBuilder>,
    ) -> HyperPlaceOrder {
        order::make_position_tp_sl(asset, is_buy, &size, Some(tp_trigger), Some(sl_trigger), builder)
    }

    fn make_withdraw(&self, amount: String, address: String, nonce: u64) -> HyperWithdrawalRequest {
        HyperWithdrawalRequest::new(amount, nonce, address)
    }

    fn make_set_referrer(&self, referrer: String) -> HyperSetReferrer {
        HyperSetReferrer::new(referrer)
    }

    fn serialize_set_referrer(&self, set_referrer: &HyperSetReferrer) -> String {
        serde_json::to_string(set_referrer).unwrap()
    }

    fn make_update_leverage(&self, asset: u32, is_cross: bool, leverage: u64) -> HyperUpdateLeverage {
        HyperUpdateLeverage::new(asset, is_cross, leverage)
    }

    fn serialize_update_leverage(&self, update_leverage: &HyperUpdateLeverage) -> String {
        serde_json::to_string(update_leverage).unwrap()
    }

    fn transfer_to_hyper_evm(&self, amount: String, time: u64, token: String) -> HyperSpotSend {
        HyperSpotSend::new(amount, HYPERCORE_EVM_BRIDGE_ADDRESS.to_string(), time, token)
    }

    fn send_spot_token_to_address(&self, amount: String, destination: String, time: u64, token: String) -> HyperSpotSend {
        HyperSpotSend::new(amount, destination, time, token)
    }

    fn serialize_spot_send(&self, spot_send: &HyperSpotSend) -> String {
        serde_json::to_string(spot_send).unwrap()
    }

    fn send_perps_usd_to_address(&self, amount: String, destination: String, time: u64) -> HyperUsdSend {
        HyperUsdSend::new(amount, destination, time)
    }

    fn serialize_usd_send(&self, usd_send: &HyperUsdSend) -> String {
        serde_json::to_string(usd_send).unwrap()
    }

    fn transfer_spot_to_perps(&self, amount: String, nonce: u64) -> HyperUsdClassTransfer {
        HyperUsdClassTransfer::new(amount, true, nonce)
    }

    fn transfer_perps_to_spot(&self, amount: String, nonce: u64) -> HyperUsdClassTransfer {
        HyperUsdClassTransfer::new(amount, false, nonce)
    }

    fn serialize_usd_class_transfer(&self, usd_class_transfer: &HyperUsdClassTransfer) -> String {
        serde_json::to_string(usd_class_transfer).unwrap()
    }

    fn make_transfer_to_staking(&self, wei: u64, nonce: u64) -> HyperCDeposit {
        HyperCDeposit::new(wei, nonce)
    }

    fn serialize_c_deposit(&self, c_deposit: &HyperCDeposit) -> String {
        serde_json::to_string(c_deposit).unwrap()
    }

    fn make_delegate(&self, validator: String, wei: u64, nonce: u64) -> HyperTokenDelegate {
        HyperTokenDelegate::new(validator, wei, false, nonce)
    }

    fn make_undelegate(&self, validator: String, wei: u64, nonce: u64) -> HyperTokenDelegate {
        HyperTokenDelegate::new(validator, wei, true, nonce)
    }

    fn serialize_token_delegate(&self, token_delegate: &HyperTokenDelegate) -> String {
        serde_json::to_string(token_delegate).unwrap()
    }

    fn build_signed_request(&self, signature: String, action: String, timestamp: u64) -> String {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_market_order_action() {
        let factory = HyperCoreModelFactory::new();
        let order = factory.make_market_order(5, true, "200.21".to_string(), "0.28".to_string(), false, None);
        let action_json = factory.serialize_order(&order);

        // Create signed request using build_signed_request
        let signature = "f3d38b1bf49efb57622bc054d115be8b8d8440b00e45610412d22ffb5ae798f93785cc770743535a79ead405b776bd5996bd62e680a10d614829bb5a733622091c";
        let timestamp = 1753576312346u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        // Parse the result and compare with expected test data
        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();

        // Load expected test data
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_open_long_order.json")).unwrap();

        // Compare the structure
        assert_eq!(parsed["action"], expected["action"]);
        assert_eq!(parsed["isFrontend"], expected["isFrontend"]);
        assert_eq!(parsed["nonce"], expected["nonce"]);
        assert_eq!(parsed["signature"]["r"], expected["signature"]["r"]);
        assert_eq!(parsed["signature"]["s"], expected["signature"]["s"]);
        assert_eq!(parsed["signature"]["v"], expected["signature"]["v"]);
    }

    #[test]
    fn test_make_market_order_short_action() {
        let actions = HyperCoreModelFactory::new();
        let order = actions.make_market_order(25, false, "3.032".to_string(), "1".to_string(), false, None);

        // Verify the structure matches the expected format for short
        assert_eq!(order.r#type, "order");
        assert_eq!(order.grouping, order::HyperGrouping::Na);
        assert_eq!(order.orders.len(), 1);

        let order_item = &order.orders[0];
        assert_eq!(order_item.asset, 25);
        assert!(!order_item.is_buy); // Short position
        assert_eq!(order_item.price, "3.032");
        assert_eq!(order_item.size, "1");
        assert!(!order_item.reduce_only);

        // Test JSON serialization for short
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["type"], "order");
        assert_eq!(json["grouping"], "na");
        assert_eq!(json["orders"][0]["a"], 25);
        assert_eq!(json["orders"][0]["b"], false); // Short
        assert_eq!(json["orders"][0]["p"], "3.032");
        assert_eq!(json["orders"][0]["s"], "1");
        assert_eq!(json["orders"][0]["r"], false);
        assert_eq!(json["orders"][0]["t"]["limit"]["tif"], "FrontendMarket");
    }

    #[test]
    fn test_make_set_referrer_action() {
        let factory = HyperCoreModelFactory::new();
        let set_referrer = factory.make_set_referrer("GEMWALLET".to_string());
        let action_json = factory.serialize_set_referrer(&set_referrer);

        // Create signed request using build_signed_request
        let signature = "750edadc6664badceff6d1cd2a96e0aed1e28b0063d9a665e6a8901983de83667872605712424e287f8d02b888ba826a872b0e89a95a50d49388d74e10c41bb31b";
        let timestamp = 1753882649539u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        // Parse the result and compare with expected test data
        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();

        // Load expected test data
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_set_referrer.json")).unwrap();

        // Compare the entire JSON structure
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_make_update_leverage_action() {
        let factory = HyperCoreModelFactory::new();
        let update_leverage = factory.make_update_leverage(25, true, 10);

        // Verify the structure
        assert_eq!(update_leverage.r#type, "updateLeverage");
        assert_eq!(update_leverage.asset, 25);
        assert!(update_leverage.is_cross);
        assert_eq!(update_leverage.leverage, 10);

        // Test JSON serialization
        let json = serde_json::to_value(&update_leverage).unwrap();
        assert_eq!(json["type"], "updateLeverage");
        assert_eq!(json["asset"], 25);
        assert_eq!(json["isCross"], true);
        assert_eq!(json["leverage"], 10);
    }

    #[test]
    fn test_make_update_leverage_isolated() {
        let factory = HyperCoreModelFactory::new();
        let update_leverage = factory.make_update_leverage(5, false, 5);

        // Verify isolated leverage
        assert_eq!(update_leverage.asset, 5);
        assert!(!update_leverage.is_cross);
        assert_eq!(update_leverage.leverage, 5);

        // Test JSON serialization for isolated
        let json = serde_json::to_value(&update_leverage).unwrap();
        assert_eq!(json["asset"], 5);
        assert_eq!(json["isCross"], false);
        assert_eq!(json["leverage"], 5);
    }

    #[test]
    fn test_transfer_to_hyper_evm() {
        let factory = HyperCoreModelFactory::new();
        let spot_send = factory.transfer_to_hyper_evm("0.1".to_string(), 1754996222238, "HYPE:0x0d01dc56dcaaca66ad901c959b4011ec".to_string());
        let action_json = factory.serialize_spot_send(&spot_send);

        let signature = "01df5d20fb1d09eed99ccf2381c1fc00e21538fcfa0babfe523bf094bb292f0825b3c888612fc55a9fdbed92828aa3c64a8c25e9b71da81b59e6fd5ddfddf6841b";
        let timestamp = 1754996222238u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_core_to_evm.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_send_spot_token_to_address() {
        let factory = HyperCoreModelFactory::new();
        let spot_send = factory.send_spot_token_to_address(
            "0.02".to_string(),
            "0x1085c5f70f7f7591d97da281a64688385455c2bd".to_string(),
            1755004027201,
            "USDC:0x6d1e7cde53ba9467b783cb7c530ce054".to_string(),
        );
        let action_json = factory.serialize_spot_send(&spot_send);

        let signature = "382d6358765ddbefb1ced7fdcd14406b8500a2b2a61332bd67ac0ce3746b9d3e5c3156b7ef4ad6d17b0ff7966e7aad1b19a4649eddcd186ad3f46013a013980d1c";
        let timestamp = 1755004027201u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_spot_send_l1.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_transfer_spot_to_perps() {
        let factory = HyperCoreModelFactory::new();
        let usd_class_transfer = factory.transfer_spot_to_perps("10".to_string(), 1754986567194);
        let action_json = factory.serialize_usd_class_transfer(&usd_class_transfer);

        let signature = "922ab18d3babc74d86c8bb0c259c121193afc57b156b512914ae81c2faad1fb316fc542cdbae9cca3984646759c9e54645a6a92e8adc597d25f0c59a23922a931b";
        let timestamp = 1754986567194u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_spot_to_perps.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_transfer_perps_to_spot() {
        let factory = HyperCoreModelFactory::new();
        let usd_class_transfer = factory.transfer_perps_to_spot("10".to_string(), 1754986301493);
        let action_json = factory.serialize_usd_class_transfer(&usd_class_transfer);

        let signature = "2a0d2571330681c146a744ce32aa31fff5ff720dff6c6e440a2724f64c99e3126c7e4296752d20f79573b3bfea00f95f092105235269e38a4bd3e987e0486b851b";
        let timestamp = 1754986301493u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_perp_to_spot.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_cancel_orders() {
        let factory = HyperCoreModelFactory::new();
        let cancels = vec![HyperCancelOrder::new(0, 133614972850), HyperCancelOrder::new(7, 133610221604)];
        let cancel_action = factory.make_cancel_orders(cancels);
        let action_json = factory.serialize_cancel_action(&cancel_action);

        let signature = "6d7f8feddf09ac204b786ff82a508134b28ba7d91ed412fef5ae0b8561ea26d31c31d3653a2ef71334e733cc81a541db2982724c785002f82e634c71c64726b01b";
        let timestamp = 1755132902800u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_cancel_orders.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_update_position_tp_sl() {
        let factory = HyperCoreModelFactory::new();
        let order = factory.make_position_tp_sl(7, false, "0.197".to_string(), "850".to_string(), "730".to_string(), None);
        let action_json = factory.serialize_order(&order);

        let signature = "e7573d3fadf28422e2068a7f477bed470bfea5627dcd0282283822250440bff0027462c40186ea0d4b50df44cf0f7b176acd21affbe03d4b0417b12cddb139b91b";
        let timestamp = 1755132472149u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_update_position_tp_sl.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_market_with_tp_sl() {
        let factory = HyperCoreModelFactory::new();
        let order = factory.make_market_with_tp_sl(
            25,
            false,
            "3.0535".to_string(),
            "5".to_string(),
            false,                   // entry order reduce_only: false
            Some("3".to_string()),   // tp_trigger: lower price for short TP
            Some("3.5".to_string()), // sl_trigger: higher price for short SL
            None,
        );
        let action_json = factory.serialize_order(&order);

        let signature = "d49f4af2a7a7037008a3fffd072914b509a685b8e3fc8c08450ff47e300b14cc1716da99ec62e121d97aac2f44c24fcdd4b64bf18ab11afe469c006552317ba61c";
        let timestamp = 1755135350327u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_market_short_tp_sl.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_market_with_tp() {
        let factory = HyperCoreModelFactory::new();
        let order = factory.make_market_with_tp_sl(25, false, "3.0535".to_string(), "5".to_string(), false, Some("3".to_string()), None, None);
        assert_eq!(order.grouping, HyperGrouping::NormalTpsl);
        assert_eq!(order.orders.len(), 2);

        let entry_order = &order.orders[0];
        assert_eq!(entry_order.asset, 25);
        assert!(!entry_order.is_buy);
        assert_eq!(entry_order.price, "3.0535");
        assert!(!entry_order.reduce_only);

        let tp_order = &order.orders[1];
        assert_eq!(tp_order.asset, 25);
        assert!(tp_order.is_buy);
        assert_eq!(tp_order.price, "3.24");
        assert!(tp_order.reduce_only);
    }

    #[test]
    fn test_market_with_sl() {
        let factory = HyperCoreModelFactory::new();
        let order = factory.make_market_with_tp_sl(25, false, "3.0535".to_string(), "5".to_string(), false, None, Some("3.5".to_string()), None);

        assert_eq!(order.grouping, HyperGrouping::NormalTpsl);
        assert_eq!(order.orders.len(), 2);

        let entry_order = &order.orders[0];
        assert_eq!(entry_order.asset, 25);
        assert!(!entry_order.is_buy);
        assert_eq!(entry_order.price, "3.0535");
        assert!(!entry_order.reduce_only);

        let sl_order = &order.orders[1];
        assert_eq!(sl_order.asset, 25);
        assert!(sl_order.is_buy);
        assert_eq!(sl_order.price, "3.78");
        assert!(sl_order.reduce_only);
    }

    #[test]
    fn test_make_c_deposit_action() {
        let factory = HyperCoreModelFactory::new();
        let c_deposit = factory.make_transfer_to_staking(10000000, 1755231476741);
        let action_json = factory.serialize_c_deposit(&c_deposit);

        let signature = "8e5d7b14d80a8a5d2334509c1f055be0ea8a78c0632ef43bd17b0f788de3538e426730e6231d72d3b6ea892b791bf68351de0c754d073bf6f2174accb4176d751c";
        let timestamp = 1755231476741u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_spot_to_stake.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_make_token_delegate_action() {
        let factory = HyperCoreModelFactory::new();
        let token_delegate = factory.make_delegate("0x5ac99df645f3414876c816caa18b2d234024b487".to_string(), 10000000, 1755231522831);
        let action_json = factory.serialize_token_delegate(&token_delegate);

        let signature = "3d16b033812211ff3b0bf7793cc628cd4db7cc273dab2264225386a158db842e36175c089b06dc245e273d7d7deedad4bd46fb5ce256a5c8de1d6a55a72580081c";
        let timestamp = 1755231522831u64;

        let signed_request = factory.build_signed_request(signature.to_string(), action_json, timestamp);

        let parsed: serde_json::Value = serde_json::from_str(&signed_request).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_stake_to_validator.json")).unwrap();

        assert_eq!(parsed, expected);
    }
}

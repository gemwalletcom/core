pub mod approve_agent;
pub mod approve_builder_fee;
pub mod order;
pub mod set_referrer;
pub mod withdrawal;

pub use approve_agent::*;
pub use approve_builder_fee::*;
pub use order::*;
pub use set_referrer::*;
pub use withdrawal::*;

pub const MAINNET: &str = "Mainnet";
pub const SIGNATURE_CHAIN_ID: &str = "0xa4b1";

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
        order::make_market_order(asset, is_buy, price, size, reduce_only, builder)
    }

    fn serialize_order(&self, order: &HyperPlaceOrder) -> String {
        serde_json::to_string(order).unwrap()
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

    fn build_signed_request(&self, signature: String, action: String, timestamp: u64) -> String {
        let sig_bytes = hex::decode(signature).unwrap();

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
}

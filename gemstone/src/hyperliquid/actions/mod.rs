pub mod approve_agent;
pub mod approve_builder_fee;
pub mod cancel_order;
pub mod place_order;
pub mod set_referrer;
pub mod withdrawal;

pub use approve_agent::*;
pub use approve_builder_fee::*;
pub use cancel_order::*;
pub use place_order::*;
pub use set_referrer::*;
pub use withdrawal::*;

pub const MAINNET: &str = "Mainnet";
pub const SIGNATURE_CHAIN_ID: &str = "0xa4b1";

#[derive(uniffi::Object)]
pub struct HyperCoreModelFactory {}

#[uniffi::export]
impl HyperCoreModelFactory {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    fn make_approve_agent(&self, name: String, address: String, nonce: u64) -> approve_agent::HyperApproveAgent {
        approve_agent::HyperApproveAgent::new(address, name, nonce)
    }

    fn make_approve_builder(&self, max_fee_rate: String, builder: String, nonce: u64) -> approve_builder_fee::HyperApproveBuilderFee {
        approve_builder_fee::HyperApproveBuilderFee::new(max_fee_rate, builder, nonce)
    }

    fn make_market_close(&self, asset: u32, price: String, size: String, reduce_only: bool) -> place_order::HyperPlaceOrder {
        place_order::make_market_close(asset, price, size, reduce_only)
    }

    fn make_market_open(&self, asset: u32, is_buy: bool, price: String, size: String, reduce_only: bool) -> place_order::HyperPlaceOrder {
        place_order::make_market_open(asset, is_buy, price, size, reduce_only)
    }

    fn make_withdraw(&self, amount: String, address: String, nonce: u64) -> withdrawal::HyperWithdrawalRequest {
        withdrawal::HyperWithdrawalRequest::new(amount, nonce, address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_market_close_action() {
        let actions = HyperCoreModelFactory::new();
        let order = actions.make_market_close(14, "3.8185".to_string(), "6.2".to_string(), true);

        // Verify the structure matches the expected format
        assert_eq!(order.action_type, "order");
        assert_eq!(order.grouping, place_order::HyperGrouping::Na);
        assert_eq!(order.orders.len(), 1);

        let order_item = &order.orders[0];
        assert_eq!(order_item.asset, 14);
        assert!(!order_item.is_buy);
        assert_eq!(order_item.price, "3.8185");
        assert_eq!(order_item.size, "6.2");
        assert!(order_item.reduce_only);

        // Check the order type structure
        match &order_item.order_type {
            place_order::HyperOrderType::Limit { limit } => {
                match limit.tif {
                    place_order::HyperTimeInForce::FrontendMarket => {
                        // This is expected for market sell
                    }
                    _ => panic!("Expected FrontendMarket TimeInForce"),
                }
            }
            _ => panic!("Expected Limit order type"),
        }

        // Test JSON serialization to ensure it matches expected structure
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["type"], "order");
        assert_eq!(json["grouping"], "na");
        assert_eq!(json["orders"][0]["a"], 14);
        assert_eq!(json["orders"][0]["b"], false);
        assert_eq!(json["orders"][0]["p"], "3.8185");
        assert_eq!(json["orders"][0]["s"], "6.2");
        assert_eq!(json["orders"][0]["r"], true);
        assert_eq!(json["orders"][0]["t"]["limit"]["tif"], "FrontendMarket");
    }

    #[test]
    fn test_make_market_open_action() {
        let actions = HyperCoreModelFactory::new();
        let order = actions.make_market_open(5, true, "200.21".to_string(), "0.28".to_string(), false);

        // Verify the structure matches the expected format
        assert_eq!(order.action_type, "order");
        assert_eq!(order.grouping, place_order::HyperGrouping::Na);
        assert_eq!(order.orders.len(), 1);

        let order_item = &order.orders[0];
        assert_eq!(order_item.asset, 5);
        assert!(order_item.is_buy);
        assert_eq!(order_item.price, "200.21");
        assert_eq!(order_item.size, "0.28");
        assert!(!order_item.reduce_only);

        // Check the order type structure
        match &order_item.order_type {
            place_order::HyperOrderType::Limit { limit } => {
                match limit.tif {
                    place_order::HyperTimeInForce::FrontendMarket => {
                        // This is expected for market buy
                    }
                    _ => panic!("Expected FrontendMarket TimeInForce"),
                }
            }
            _ => panic!("Expected Limit order type"),
        }

        // Test JSON serialization to ensure it matches expected structure
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["type"], "order");
        assert_eq!(json["grouping"], "na");
        assert_eq!(json["orders"][0]["a"], 5);
        assert_eq!(json["orders"][0]["b"], true);
        assert_eq!(json["orders"][0]["p"], "200.21");
        assert_eq!(json["orders"][0]["s"], "0.28");
        assert_eq!(json["orders"][0]["r"], false);
        assert_eq!(json["orders"][0]["t"]["limit"]["tif"], "FrontendMarket");
    }

    #[test]
    fn test_make_market_open_short_action() {
        let actions = HyperCoreModelFactory::new();
        let order = actions.make_market_open(25, false, "3.032".to_string(), "1".to_string(), false);

        // Verify the structure matches the expected format for short
        assert_eq!(order.action_type, "order");
        assert_eq!(order.grouping, place_order::HyperGrouping::Na);
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

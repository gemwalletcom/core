use serde_json::Value;

use super::{eip712, hahser::action_hash, models::PhantomAgent};
use crate::hyperliquid::actions;

#[derive(uniffi::Object)]
pub struct HyperCore {}

impl HyperCore {
    fn l1_action_typed_data(&self, action: Value, nonce: u64) -> String {
        let hash = action_hash(&action, None, nonce, None).unwrap();
        let phantom_agent = PhantomAgent::new(hash);
        eip712::create_l1_eip712_json(&phantom_agent)
    }
}

#[uniffi::export]
impl HyperCore {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    // L1 payload
    fn place_order_typed_data(&self, order: actions::HyperPlaceOrder, nonce: u64) -> String {
        let action_value = serde_json::to_value(&order).unwrap();
        self.l1_action_typed_data(action_value, nonce)
    }

    // L1 payload
    fn set_referrer_typed_data(&self, referrer: actions::HyperSetReferrer, nonce: u64) -> String {
        let action_value = serde_json::to_value(&referrer).unwrap();
        self.l1_action_typed_data(action_value, nonce)
    }

    // User signed payload
    fn withdrawal_request_typed_data(&self, request: actions::HyperWithdrawalRequest) -> String {
        let action_value = serde_json::to_value(&request).unwrap();
        eip712::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:Withdraw", eip712::withdraw_types())
    }

    // User signed payload
    fn approve_agent_typed_data(&self, agent: actions::HyperApproveAgent) -> String {
        let action_value = serde_json::to_value(&agent).unwrap();
        eip712::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:ApproveAgent", eip712::approve_agent_types())
    }

    // User signed payload
    fn approve_builder_fee_typed_data(&self, fee: actions::HyperApproveBuilderFee) -> String {
        let action_value = serde_json::to_value(&fee).unwrap();
        eip712::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:ApproveBuilderFee", eip712::approve_builder_fee_types())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyperliquid::actions;

    #[test]
    fn test_action_open_long_matches_test_data() {
        let order = actions::make_market_order(5, true, "200.21".to_string(), "0.28".to_string(), false);
        let generated_action: serde_json::Value = serde_json::to_value(&order).unwrap();

        // Load expected data from test file
        let test_data: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_open_long_order.json")).unwrap();
        let expected_action = &test_data["action"];

        assert_eq!(generated_action, *expected_action);
    }

    #[test]
    fn test_action_open_short_matches_test_data() {
        let order = actions::make_market_order(25, false, "3.032".to_string(), "1".to_string(), false);
        let generated_action: serde_json::Value = serde_json::to_value(&order).unwrap();

        // Load expected data from test file
        let test_data: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_open_short_order.json")).unwrap();
        let expected_action = &test_data["action"];

        assert_eq!(generated_action, *expected_action);
    }

    #[test]
    fn test_eip712_approve_agent_matches_test_data() {
        let hypercore = HyperCore::new();
        let agent = actions::HyperApproveAgent::new("0xbec81216a5edeaed508709d8526078c750e307ad".to_string(), "".to_string(), 1753576844319);

        let eip712_json = hypercore.approve_agent_typed_data(agent);

        // Pretty print the generated JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let pretty_generated = serde_json::to_string_pretty(&parsed).unwrap();

        // Load expected test data
        let expected = include_str!("../test/hl_eip712_approve_agent.json").trim();

        assert_eq!(pretty_generated, expected);
    }

    #[test]
    fn test_eip712_withdrawal_matches_test_data() {
        let hypercore = HyperCore::new();
        let withdrawal = actions::HyperWithdrawalRequest::new("2".to_string(), 1753577591421, "0x514bcb1f9aabb904e6106bd1052b66d2706dbbb7".to_string());

        let eip712_json = hypercore.withdrawal_request_typed_data(withdrawal);

        // Pretty print the generated JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let pretty_generated = serde_json::to_string_pretty(&parsed).unwrap();

        // Load expected test data
        let expected = include_str!("../test/hl_eip712_withdraw.json").trim();

        assert_eq!(pretty_generated, expected);
    }

    #[test]
    fn test_l1_action_hash() {
        // https://github.com/hyperliquid-dex/hyperliquid-python-sdk/blob/master/tests/signing_test.py#L20
        // ETH buy order, sz=0.0147, limit_px=1670.1, asset=4, is_buy=true, reduce_only=false, IoC
        use actions::{HyperGrouping, HyperLimitOrder, HyperOrder, HyperOrderType, HyperPlaceOrder, HyperTimeInForce};

        let order = HyperPlaceOrder::new(
            vec![HyperOrder {
                asset: 4,
                is_buy: true,
                price: "1670.1".to_string(),
                reduce_only: false,
                size: "0.0147".to_string(),
                order_type: HyperOrderType::Limit {
                    limit: HyperLimitOrder::new(HyperTimeInForce::ImmediateOrCancel),
                },
                client_order_id: None,
            }],
            HyperGrouping::Na,
            None,
        );

        let action_value = serde_json::to_value(&order).unwrap();
        let nonce = 1677777606040u64;
        let hash = action_hash(&action_value, None, nonce, None).unwrap();
        let expected_connection_id = "0x0fcbeda5ae3c4950a548021552a4fea2226858c4453571bf3f24ba017eac2908";
        let phantom_agent = PhantomAgent::new(hash.clone());

        assert_eq!(phantom_agent.source, "a");
        assert_eq!(phantom_agent.connection_id, expected_connection_id);

        assert_eq!(action_value["type"], "order");
        assert_eq!(action_value["grouping"], "na");
        assert_eq!(action_value["orders"][0]["a"], 4);
        assert_eq!(action_value["orders"][0]["b"], true);
        assert_eq!(action_value["orders"][0]["p"], "1670.1");
        assert_eq!(action_value["orders"][0]["s"], "0.0147");
        assert_eq!(action_value["orders"][0]["r"], false);
        assert_eq!(action_value["orders"][0]["t"]["limit"]["tif"], "Ioc");
    }

    #[test]
    fn test_address_lowercasing_in_actions() {
        // Test that addresses are properly lowercased in action constructors
        let uppercase_address = "0xBEC81216A5EDEAED508709D8526078C750E307AD";
        let expected_lowercase = "0xbec81216a5edeaed508709d8526078c750e307ad";

        // Test withdrawal request
        let withdrawal = actions::HyperWithdrawalRequest::new("2".to_string(), 1753577591421, uppercase_address.to_string());
        assert_eq!(withdrawal.destination, expected_lowercase);

        // Test approve agent
        let agent = actions::HyperApproveAgent::new(uppercase_address.to_string(), "test".to_string(), 1753576844319);
        assert_eq!(agent.agent_address, expected_lowercase);

        // Test approve builder fee
        let fee = actions::HyperApproveBuilderFee::new("0.001".to_string(), uppercase_address.to_string(), 1753576844319);
        assert_eq!(fee.builder, expected_lowercase);
    }

    #[test]
    fn test_user_signed_action_fields_added_during_encoding() {
        // Test that hyperliquidChain and signatureChainId are added during encoding
        let agent = actions::HyperApproveAgent::new("0xbec81216a5edeaed508709d8526078c750e307ad".to_string(), "".to_string(), 1753576844319);

        let hypercore = HyperCore::new();
        let eip712_json = hypercore.approve_agent_typed_data(agent);

        // Parse the JSON to verify the fields are present
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let message = &parsed["message"];

        assert_eq!(message["signatureChainId"], "0xa4b1");
        assert_eq!(message["hyperliquidChain"], "Mainnet");

        // Verify original action fields are present
        assert_eq!(message["agentAddress"], "0xbec81216a5edeaed508709d8526078c750e307ad");
        assert_eq!(message["agentName"], "");
        assert_eq!(message["nonce"], 1753576844319u64);
    }
}

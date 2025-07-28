pub mod actions;
mod encoding;

#[derive(uniffi::Object)]
pub struct HyperCore {}

#[uniffi::export]
impl HyperCore {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    // L1 payload
    fn encode_place_order(&self, order: actions::HyperPlaceOrder, nonce: u64) -> String {
        let action_value = serde_json::to_value(&order).unwrap();
        let hash = encoding::action_hash(&action_value, None, nonce, None).unwrap();
        let phantom_agent = encoding::PhantomAgent::new(hash);
        encoding::create_l1_eip712_json(&phantom_agent)
    }

    // L1 payload
    fn encode_cancel_order(&self, order: actions::HyperCancelOrder, nonce: u64) -> String {
        let action_value = serde_json::to_value(&order).unwrap();
        let hash = encoding::action_hash(&action_value, None, nonce, None).unwrap();
        let phantom_agent = encoding::PhantomAgent::new(hash);
        encoding::create_l1_eip712_json(&phantom_agent)
    }

    // User signed payload
    fn encode_withdrawal_request(&self, request: actions::HyperWithdrawalRequest) -> String {
        let action_value = serde_json::to_value(&request).unwrap();
        encoding::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:Withdraw", encoding::withdraw_types())
    }

    // User signed payload
    fn encode_approve_agent(&self, agent: actions::HyperApproveAgent) -> String {
        let action_value = serde_json::to_value(&agent).unwrap();
        encoding::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:ApproveAgent", encoding::approve_agent_types())
    }

    // User signed payload
    fn encode_approve_builder_fee(&self, fee: actions::HyperApproveBuilderFee) -> String {
        let action_value = serde_json::to_value(&fee).unwrap();
        encoding::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:ApproveBuilderFee", encoding::approve_builder_fee_types())
    }

    // User signed payload
    fn encode_set_referrer(&self, referrer: actions::HyperSetReferrer) -> String {
        let action_value = serde_json::to_value(&referrer).unwrap();
        encoding::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:SetReferrer", encoding::set_referrer_types())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyperliquid::actions;

    #[test]
    fn test_action_close_order_matches_test_data() {
        let order = actions::make_market_close(14, "3.8185".to_string(), "6.2".to_string(), true);
        let generated_action: serde_json::Value = serde_json::to_value(&order).unwrap();
        
        // Load expected data from test file
        let test_data: serde_json::Value = serde_json::from_str(include_str!("test/hl_action_close_order.json")).unwrap();
        let expected_action = &test_data["action"];
        
        assert_eq!(generated_action, *expected_action);
    }

    #[test]
    fn test_action_open_long_matches_test_data() {
        let order = actions::make_market_open(5, true, "200.21".to_string(), "0.28".to_string(), false);
        let generated_action: serde_json::Value = serde_json::to_value(&order).unwrap();
        
        // Load expected data from test file
        let test_data: serde_json::Value = serde_json::from_str(include_str!("test/hl_action_open_long_order.json")).unwrap();
        let expected_action = &test_data["action"];
        
        assert_eq!(generated_action, *expected_action);
    }

    #[test]
    fn test_action_open_short_matches_test_data() {
        let order = actions::make_market_open(25, false, "3.032".to_string(), "1".to_string(), false);
        let generated_action: serde_json::Value = serde_json::to_value(&order).unwrap();
        
        // Load expected data from test file
        let test_data: serde_json::Value = serde_json::from_str(include_str!("test/hl_action_open_short_order.json")).unwrap();
        let expected_action = &test_data["action"];
        
        assert_eq!(generated_action, *expected_action);
    }

    #[test]
    fn test_eip712_approve_agent_matches_test_data() {
        let hypercore = HyperCore::new();
        let agent = actions::HyperApproveAgent::new(
            "0xbec81216a5edeaed508709d8526078c750e307ad".to_string(),
            "".to_string(),
            1753576844319
        );
        
        let eip712_json = hypercore.encode_approve_agent(agent);
        
        // Pretty print the generated JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let pretty_generated = serde_json::to_string_pretty(&parsed).unwrap();
        
        // Load expected test data
        let expected = include_str!("test/hl_eip712_approve_agent.json").trim();
        
        assert_eq!(pretty_generated, expected);
    }

    #[test]
    fn test_eip712_withdrawal_matches_test_data() {
        let hypercore = HyperCore::new();
        let withdrawal = actions::HyperWithdrawalRequest::new(
            "2".to_string(),
            1753577591421,
            "0x514bcb1f9aabb904e6106bd1052b66d2706dbbb7".to_string()
        );
        
        let eip712_json = hypercore.encode_withdrawal_request(withdrawal);
        
        // Pretty print the generated JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let pretty_generated = serde_json::to_string_pretty(&parsed).unwrap();
        
        // Load expected test data
        let expected = include_str!("test/hl_eip712_withdraw.json").trim();
        
        assert_eq!(pretty_generated, expected);
    }
}

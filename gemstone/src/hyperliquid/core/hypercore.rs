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

    fn spot_send_typed_data(&self, spot_send: actions::HyperSpotSend) -> String {
        let action_value = serde_json::to_value(&spot_send).unwrap();
        eip712::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:SpotSend", eip712::spot_send_types())
    }

    fn usd_class_transfer_typed_data(&self, usd_class_transfer: actions::HyperUsdClassTransfer) -> String {
        let action_value = serde_json::to_value(&usd_class_transfer).unwrap();
        eip712::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:UsdClassTransfer", eip712::usd_class_transfer_types())
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

    // L1 payload
    fn update_leverage_typed_data(&self, update_leverage: actions::HyperUpdateLeverage, nonce: u64) -> String {
        let action_value = serde_json::to_value(&update_leverage).unwrap();
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

    fn transfer_to_hyper_evm_typed_data(&self, spot_send: actions::HyperSpotSend) -> String {
        self.spot_send_typed_data(spot_send)
    }

    fn send_spot_token_to_address_typed_data(&self, spot_send: actions::HyperSpotSend) -> String {
        self.spot_send_typed_data(spot_send)
    }

    fn send_perps_usd_to_address_typed_data(&self, usd_send: actions::HyperUsdSend) -> String {
        let action_value = serde_json::to_value(&usd_send).unwrap();
        eip712::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:UsdSend", eip712::usd_send_types())
    }

    fn transfer_spot_to_perps_typed_data(&self, usd_class_transfer: actions::HyperUsdClassTransfer) -> String {
        self.usd_class_transfer_typed_data(usd_class_transfer)
    }

    fn transfer_perps_to_spot_typed_data(&self, usd_class_transfer: actions::HyperUsdClassTransfer) -> String {
        self.usd_class_transfer_typed_data(usd_class_transfer)
    }

    // User signed payload
    fn c_deposit_typed_data(&self, c_deposit: actions::HyperCDeposit) -> String {
        let action_value = serde_json::to_value(&c_deposit).unwrap();
        eip712::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:CDeposit", eip712::c_deposit_types())
    }

    // User signed payload
    fn token_delegate_typed_data(&self, token_delegate: actions::HyperTokenDelegate) -> String {
        let action_value = serde_json::to_value(&token_delegate).unwrap();
        eip712::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:TokenDelegate", eip712::token_delegate_types())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyperliquid::actions;

    #[test]
    fn test_action_open_long() {
        let order = actions::make_market_order(5, true, "200.21", "0.28", false, None);
        let generated_action: serde_json::Value = serde_json::to_value(&order).unwrap();

        // Load expected data from test file
        let test_data: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_open_long_order.json")).unwrap();
        let expected_action = &test_data["action"];

        assert_eq!(generated_action, *expected_action);
    }

    #[test]
    fn test_action_open_short() {
        let order = actions::make_market_order(25, false, "3.032", "1", false, None);
        let generated_action: serde_json::Value = serde_json::to_value(&order).unwrap();

        // Load expected data from test file
        let test_data: serde_json::Value = serde_json::from_str(include_str!("../test/hl_action_open_short_order.json")).unwrap();
        let expected_action = &test_data["action"];

        assert_eq!(generated_action, *expected_action);
    }

    #[test]
    fn test_eip712_approve_agent() {
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
    fn test_eip712_withdrawal() {
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

    #[test]
    fn test_update_leverage_typed_data() {
        let hypercore = HyperCore::new();
        let update_leverage = actions::HyperUpdateLeverage::new(25, true, 10);
        let nonce = 1753577591421u64;

        let eip712_json = hypercore.update_leverage_typed_data(update_leverage, nonce);

        // Parse the JSON to verify structure
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();

        // Verify EIP712 structure is present
        assert!(parsed["types"].is_object());
        assert!(parsed["domain"].is_object());
        assert!(parsed["message"].is_object());
        assert_eq!(parsed["primaryType"], "Agent");

        // Verify the action was properly serialized
        let action_value = serde_json::to_value(actions::HyperUpdateLeverage::new(25, true, 10)).unwrap();
        assert_eq!(action_value["type"], "updateLeverage");
        assert_eq!(action_value["asset"], 25);
        assert_eq!(action_value["isCross"], true);
        assert_eq!(action_value["leverage"], 10);
    }

    #[test]
    fn test_eip712_spot_send_core_to_evm() {
        let hypercore = HyperCore::new();
        let spot_send = actions::HyperSpotSend::new(
            "0.1".to_string(),
            "0x2222222222222222222222222222222222222222".to_string(),
            1754996222238,
            "HYPE:0x0d01dc56dcaaca66ad901c959b4011ec".to_string(),
        );

        let eip712_json = hypercore.transfer_to_hyper_evm_typed_data(spot_send);

        // Parse both generated and expected JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_eip712_core_to_evm.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_eip712_spot_send_l1() {
        let hypercore = HyperCore::new();
        let spot_send = actions::HyperSpotSend::new(
            "0.02".to_string(),
            "0x1085c5f70f7f7591d97da281a64688385455c2bd".to_string(),
            1755004027201,
            "USDC:0x6d1e7cde53ba9467b783cb7c530ce054".to_string(),
        );

        let eip712_json = hypercore.send_spot_token_to_address_typed_data(spot_send);

        // Parse both generated and expected JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_eip712_spot_send_l1.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_eip712_usd_send() {
        let hypercore = HyperCore::new();
        let usd_send = actions::HyperUsdSend::new("1".to_string(), "0xe51d0862078098c84346b6203b50b996f7dafe28".to_string(), 1754987223323);

        let eip712_json = hypercore.send_perps_usd_to_address_typed_data(usd_send);

        // Parse both generated and expected JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_eip712_perp_send_l1.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_eip712_usd_class_transfer_perp_to_spot() {
        let hypercore = HyperCore::new();
        let usd_class_transfer = actions::HyperUsdClassTransfer::new(
            "10".to_string(),
            false, // perp to spot
            1754986301493,
        );

        let eip712_json = hypercore.transfer_perps_to_spot_typed_data(usd_class_transfer);

        // Parse both generated and expected JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_eip712_perp_to_spot.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_eip712_usd_class_transfer_spot_to_perp_structure() {
        // Test the spot to perp transfer structure (no corresponding test file yet)
        let hypercore = HyperCore::new();
        let usd_class_transfer = actions::HyperUsdClassTransfer::new(
            "10".to_string(),
            true, // spot to perp
            1754986567194,
        );

        let eip712_json = hypercore.transfer_spot_to_perps_typed_data(usd_class_transfer);

        // Parse and verify structure
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();

        // Verify domain
        assert_eq!(parsed["domain"]["name"], "HyperliquidSignTransaction");
        assert_eq!(parsed["domain"]["version"], "1");
        assert_eq!(parsed["primaryType"], "HyperliquidTransaction:UsdClassTransfer");

        // Verify message
        assert_eq!(parsed["message"]["type"], "usdClassTransfer");
        assert_eq!(parsed["message"]["amount"], "10");
        assert_eq!(parsed["message"]["toPerp"], true);
        assert_eq!(parsed["message"]["nonce"], 1754986567194u64);
        assert_eq!(parsed["message"]["signatureChainId"], "0xa4b1");
        assert_eq!(parsed["message"]["hyperliquidChain"], "Mainnet");
    }

    #[test]
    fn test_eip712_c_deposit() {
        let hypercore = HyperCore::new();
        let c_deposit = actions::HyperCDeposit::new(10000000, 1755231476741);

        let eip712_json = hypercore.c_deposit_typed_data(c_deposit);

        // Parse both generated and expected JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_eip712_spot_to_stake_balance.json")).unwrap();

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_eip712_token_delegate() {
        let hypercore = HyperCore::new();
        let token_delegate = actions::HyperTokenDelegate::new("0x5ac99df645f3414876c816caa18b2d234024b487".to_string(), 10000000, false, 1755231522831);

        let eip712_json = hypercore.token_delegate_typed_data(token_delegate);

        // Parse both generated and expected JSON for comparison
        let parsed: serde_json::Value = serde_json::from_str(&eip712_json).unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!("../test/hl_eip712_stake_to_validator.json")).unwrap();

        assert_eq!(parsed, expected);
    }
}

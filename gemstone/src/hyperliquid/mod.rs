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
        encoding::create_user_signed_eip712_json(&action_value, "HyperliquidTransaction:Withdraw3", encoding::withdraw_types())
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

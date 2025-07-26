pub mod actions;

#[derive(uniffi::Object)]
pub struct HyperCore {}

#[uniffi::export]
impl HyperCore {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    // return full eip712 message
    fn encode_place_order(&self, _order: actions::HyperPlaceOrder) -> String {
        todo!()
    }

    fn encode_cancel_order(&self, _order: actions::HyperCancelOrder) -> String {
        todo!()
    }

    fn encode_withdrawal_request(&self, _order: actions::HyperWithdrawalRequest) -> String {
        todo!()
    }

    fn encode_approve_agent(&self, _agent: actions::HyperApproveAgent) -> String {
        todo!()
    }

    fn encode_approve_builder_fee(&self, _fee: actions::HyperApproveBuilderFee) -> String {
        todo!()
    }
}

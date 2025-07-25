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
    fn encode_place_order(&self, _order: actions::PlaceOrder) -> String {
        todo!()
    }

    fn encode_cancel_order(&self, _order: actions::CancelOrder) -> String {
        todo!()
    }

    fn encode_withdrawal_request(&self, _order: actions::WithdrawalRequest) -> String {
        todo!()
    }
}

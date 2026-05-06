#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use crate::models::{Trace, TraceAction, TraceResponse, TransactionMessage};
#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::TonClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV";
#[cfg(test)]
pub const TEST_TRANSACTION_ID: &str = "gyjq/7IJ5KpSvZlnwixaS3RjI2xk1+5pup0k++S/yXY=";

#[cfg(test)]
impl TraceResponse {
    pub fn mock(transaction: TransactionMessage, is_incomplete: bool, actions: Vec<TraceAction>) -> Self {
        Self {
            traces: vec![Trace {
                is_incomplete,
                actions,
                transactions_order: vec![transaction.hash.clone()],
                transactions: HashMap::from([(transaction.hash.clone(), transaction)]),
            }],
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_ton_test_client() -> TonClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.ton.url, reqwest::Client::new());
    TonClient::new(reqwest_client)
}

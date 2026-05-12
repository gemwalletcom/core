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
pub const TEST_TRANSACTION_HEX_HASH: &str = "8328eaffb209e4aa52bd9967c22c5a4b7463236c64d7ee69ba9d24fbe4bfc976";
#[cfg(test)]
pub const FAILED_SWAP_MESSAGE_HASH: &str = "cf2fc2efd8d6f6b018f949b8f07e7e4b898a34a8bd422fcffb76bdc6e947b7e7";
#[cfg(test)]
pub const FAILED_SWAP_ROOT_TRANSACTION_HASH: &str = "L5Egpf9I3suIl6CdddcmMS44geWLFKgHi3EbBDz7qy8=";
#[cfg(test)]
pub const FAILED_SWAP_ROOT_TRANSACTION_HEX_HASH: &str = "2f9120a5ff48decb8897a09d75d726312e3881e58b14a8078b711b043cfbab2f";
#[cfg(test)]
pub const SUCCESS_SWAP_MESSAGE_HASH: &str = "e993d4c13053978b6265157561c454ef731274d836e3139ed64fdf58b6635bf7";
#[cfg(test)]
pub const SUCCESS_SWAP_ROOT_TRANSACTION_HASH: &str = "6ZPUwTBTl4tiZRV1YcRU73MSdNg24xOe1k/fWLZjW/c=";
#[cfg(test)]
pub const SUCCESS_SWAP_ROOT_TRANSACTION_HEX_HASH: &str = "e993d4c13053978b6265157561c454ef731274d836e3139ed64fdf58b6635bf7";

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

    pub fn mock_block_traces() -> Self {
        serde_json::from_str(include_str!("../../testdata/block_traces.json")).unwrap()
    }

    pub fn mock_block_trace(index: usize) -> Self {
        let traces = Self::mock_block_traces();

        TraceResponse {
            traces: vec![traces.traces[index].clone()],
        }
    }

    pub fn mock_jetton_swap() -> Self {
        serde_json::from_str(include_str!("../../testdata/jetton_swap_trace.json")).unwrap()
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_ton_test_client() -> TonClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.ton.url, reqwest::Client::new());
    TonClient::new(reqwest_client)
}

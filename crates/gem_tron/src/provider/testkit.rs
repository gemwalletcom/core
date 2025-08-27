#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::TronClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> TronClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://api.trongrid.io".to_string(), reqwest::Client::new());
    TronClient::new(reqwest_client)
}

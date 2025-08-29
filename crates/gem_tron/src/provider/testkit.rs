#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::TronClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> TronClient<ReqwestClient> {
    use crate::rpc::trongrid::client::TronGridClient;
    let url = "https://api.trongrid.io";
    let reqwest_client = ReqwestClient::new(url.to_string(), reqwest::Client::new());
    let trongrid_client = TronGridClient::new(reqwest_client.clone(), url.to_string(), "".to_string());
    TronClient::new(reqwest_client, trongrid_client)
}

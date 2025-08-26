#[cfg(all(test, feature = "integration_tests"))]
use crate::rpc::client::TonClient;
#[cfg(all(test, feature = "integration_tests"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "integration_tests"))]
pub const TEST_ADDRESS: &str = "EQD-cvR0Nz6XAyRBpDeRqKHEQIAMp2lHFKBWw9bZ-iiERTXn";

#[cfg(all(test, feature = "integration_tests"))]
pub fn create_ton_test_client() -> TonClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://toncenter.com".to_string(), reqwest::Client::new());
    TonClient::new(reqwest_client)
}

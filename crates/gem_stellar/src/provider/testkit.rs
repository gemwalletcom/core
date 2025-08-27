#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::StellarClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> StellarClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://horizon.stellar.org".to_string(), reqwest::Client::new());
    StellarClient::new(reqwest_client)
}

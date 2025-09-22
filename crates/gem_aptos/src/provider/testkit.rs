#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::AptosClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_aptos_test_client() -> AptosClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.aptos.url, reqwest::Client::new());
    AptosClient::new(reqwest_client)
}

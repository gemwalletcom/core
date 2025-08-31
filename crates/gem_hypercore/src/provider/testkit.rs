#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::HyperCoreClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "0x5ac99df645f3414876c816caa18b2d234024b487";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_hypercore_test_client() -> HyperCoreClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://api.hyperliquid.xyz".to_string(), reqwest::Client::new());
    HyperCoreClient::new(reqwest_client)
}
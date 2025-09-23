#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::HyperCoreClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "0x5ac99df645f3414876c816caa18b2d234024b487";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_hypercore_test_client() -> HyperCoreClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.hypercore.url, reqwest::Client::new());
    HyperCoreClient::new(reqwest_client)
}

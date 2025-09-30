#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::HyperCoreClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
#[cfg(all(test, feature = "chain_integration_tests"))]
pub const USDC_TOKEN_ID: &str = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_hypercore_test_client() -> HyperCoreClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.hypercore.url, reqwest::Client::new());
    HyperCoreClient::new(reqwest_client)
}

#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::PolkadotClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_polkadot_test_client() -> PolkadotClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.polkadot.url, reqwest::Client::new());
    PolkadotClient::new(reqwest_client)
}

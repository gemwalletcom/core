#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::TonClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_ton_test_client() -> TonClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.ton.url, reqwest::Client::new());
    TonClient::new(reqwest_client)
}

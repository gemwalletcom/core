#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::XRPClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "rnZmVGX6f4pUYyS4oXYJzoLdRojQV8y297";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS_EMPTY: &str = "rPGZTtsiBXS8izwJcktUmxtzZSic1jbpLi";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_xrp_test_client() -> XRPClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.xrp.url, reqwest::Client::new());
    XRPClient::new(reqwest_client)
}

#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::XRPClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "rnZmVGX6f4pUYyS4oXYJzoLdRojQV8y297";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_xrp_test_client() -> XRPClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://s1.ripple.com:51234/".to_string(), reqwest::Client::new());
    XRPClient::new(reqwest_client)
}

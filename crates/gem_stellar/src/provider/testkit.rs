#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::StellarClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "GAN2JTIWVKGZIDN5R2AFYLUV4IUXLBG3MQA3R5ECIIM5RUYT74Y3LDOP";
#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_EMPTY_ADDRESS: &str = "GBUUVZ2XQZGVPQ2IAWDTOJ3Z2UZC23I7MEAC2VRP7VCTNFOZDGCJJMEI";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> StellarClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.stellar.url, reqwest::Client::new());
    StellarClient::new(reqwest_client)
}

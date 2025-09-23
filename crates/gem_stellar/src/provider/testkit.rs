#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::StellarClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37";
#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_EMPTY_ADDRESS: &str = "GBRR4W4ATU4V6DAR5A3EBSNJM7DX3JSNBPR3ZBRPUWDO4SCWPL7Z6IIN";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> StellarClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.stellar.url, reqwest::Client::new());
    StellarClient::new(reqwest_client)
}

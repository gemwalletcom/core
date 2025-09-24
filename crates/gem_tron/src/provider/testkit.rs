#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::TronClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "TFdTEn9dJuqh351y8fyJ3eMmghFsZNwakb";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> TronClient<ReqwestClient> {
    use crate::rpc::trongrid::client::TronGridClient;
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.tron.url, reqwest::Client::new());
    let trongrid_client = TronGridClient::new(reqwest_client.clone(), settings.trongrid.key.secret);
    TronClient::new(reqwest_client, trongrid_client)
}

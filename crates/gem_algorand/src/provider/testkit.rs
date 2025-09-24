#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::AlgorandClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "RXIOUIR5IGFZMIZ7CR7FJXDYY4JI7NZG5UCWCZZNWXUPFJRLG6K6X5ITXM";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_algorand_test_client() -> AlgorandClient<ReqwestClient> {
    use crate::rpc::AlgorandClientIndexer;

    let settings = get_test_settings();
    let client = reqwest::Client::new();
    let reqwest_client = ReqwestClient::new(settings.chains.algorand.url, client.clone());
    AlgorandClient::new(
        reqwest_client.clone(),
        AlgorandClientIndexer::new(ReqwestClient::new(
            settings.chains.algorand.archive_url.expect("Missing archive URL"),
            client.clone(),
        )),
    )
}

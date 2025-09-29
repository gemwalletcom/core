#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::CosmosClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use primitives::chain_cosmos::CosmosChain;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;
#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "cosmos1cvh8mpz04az0x7vht6h6ekksg8wd650r39ltwj";
#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_EMPTY_ADDRESS: &str = "cosmos19xv76hwfjzf286we9q8ssce4v67h378vfnxvga";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_osmosis_test_client() -> CosmosClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.osmosis.url, reqwest::Client::new());
    CosmosClient::new(CosmosChain::Osmosis, reqwest_client)
}

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_cosmos_test_client() -> CosmosClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.cosmos.url, reqwest::Client::new());
    CosmosClient::new(CosmosChain::Cosmos, reqwest_client)
}

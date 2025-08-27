#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::CosmosClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use primitives::chain_cosmos::CosmosChain;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_osmosis_test_client() -> CosmosClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://lcd.osmosis.zone".to_string(), reqwest::Client::new());
    CosmosClient::new(CosmosChain::Osmosis, reqwest_client)
}

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_cosmos_test_client() -> CosmosClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://cosmos-rest.publicnode.com".to_string(), reqwest::Client::new());
    CosmosClient::new(CosmosChain::Cosmos, reqwest_client)
}

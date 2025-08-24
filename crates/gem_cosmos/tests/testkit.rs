use gem_client::ReqwestClient;
use gem_cosmos::rpc::client::CosmosClient;
use primitives::chain_cosmos::CosmosChain;

pub fn create_osmosis_test_client() -> CosmosClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://lcd.osmosis.zone".to_string(), reqwest::Client::new());
    CosmosClient::new(CosmosChain::Osmosis, reqwest_client, "https://lcd.osmosis.zone".to_string())
}

pub fn create_cosmos_test_client() -> CosmosClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://cosmos-rest.publicnode.com".to_string(), reqwest::Client::new());
    CosmosClient::new(CosmosChain::Cosmos, reqwest_client, "https://cosmos-rest.publicnode.com".to_string())
}

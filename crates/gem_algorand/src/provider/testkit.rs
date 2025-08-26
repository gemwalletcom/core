#[cfg(all(test, feature = "integration_tests"))]
use crate::rpc::client::AlgorandClient;
#[cfg(all(test, feature = "integration_tests"))]
use crate::rpc::client_indexer::AlgorandClientIndexer;
#[cfg(all(test, feature = "integration_tests"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "integration_tests"))]
pub const TEST_ADDRESS: &str = "RXIOUIR5IGFZMIZ7CR7FJXDYY4JI7NZG5UCWCZZNWXUPFJRLG6K6X5ITXM";

#[cfg(all(test, feature = "integration_tests"))]
pub fn create_algorand_test_client() -> AlgorandClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://mainnet-api.algonode.cloud".to_string(), reqwest::Client::new());
    AlgorandClient::new(reqwest_client)
}

#[cfg(all(test, feature = "integration_tests"))]
pub fn create_algorand_test_indexer_client() -> AlgorandClientIndexer<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://mainnet-idx.algonode.cloud".to_string(), reqwest::Client::new());
    AlgorandClientIndexer::new(reqwest_client)
}

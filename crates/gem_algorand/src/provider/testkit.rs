#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use crate::rpc::client::AlgorandClient;
#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub const TEST_ADDRESS: &str = "RXIOUIR5IGFZMIZ7CR7FJXDYY4JI7NZG5UCWCZZNWXUPFJRLG6K6X5ITXM";

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_algorand_test_client() -> AlgorandClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://mainnet-idx.algonode.cloud".to_string(), reqwest::Client::new());
    AlgorandClient::new(reqwest_client)
}

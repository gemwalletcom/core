#[cfg(all(test, feature = "integration_tests"))]
use crate::SuiClient;
#[cfg(all(test, feature = "integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "integration_tests"))]
use gem_jsonrpc::client::JsonRpcClient;

#[cfg(all(test, feature = "integration_tests"))]
pub const TEST_ADDRESS: &str = "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6";

#[cfg(all(test, feature = "integration_tests"))]
pub fn create_sui_test_client() -> SuiClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://fullnode.mainnet.sui.io:443".to_string(), reqwest::Client::new());
    SuiClient::new(JsonRpcClient::new(reqwest_client))
}

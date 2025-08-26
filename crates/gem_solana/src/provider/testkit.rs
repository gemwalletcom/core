#[cfg(all(test, feature = "integration_tests"))]
use crate::rpc::client::SolanaClient;
#[cfg(all(test, feature = "integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "integration_tests"))]
use gem_jsonrpc::JsonRpcClient;

#[cfg(all(test, feature = "integration_tests"))]
pub const TEST_ADDRESS: &str = "6sbzC1eH4FTujJXWj51eQe25cYvr4xfXbJ1vAj7j2k5J";

#[cfg(all(test, feature = "integration_tests"))]
pub fn create_test_client() -> SolanaClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://api.mainnet-beta.solana.com".to_string(), reqwest::Client::new());
    let rpc_client = JsonRpcClient::new(reqwest_client);
    SolanaClient::new(rpc_client)
}

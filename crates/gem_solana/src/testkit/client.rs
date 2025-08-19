#[cfg(all(test, feature = "reqwest", feature = "rpc"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "reqwest", feature = "rpc"))]
use gem_jsonrpc::client::JsonRpcClient;
#[cfg(all(test, feature = "reqwest", feature = "rpc"))]
use crate::rpc::client::SolanaClient;

#[cfg(all(test, feature = "reqwest", feature = "rpc"))]
pub fn create_test_client() -> SolanaClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://api.mainnet-beta.solana.com".to_string(), reqwest::Client::new());
    let rpc_client = JsonRpcClient::new(reqwest_client);
    SolanaClient::new(rpc_client)
}
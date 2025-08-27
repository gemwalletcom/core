#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::SolanaClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_jsonrpc::JsonRpcClient;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "8kvn29Nd9St6icquZFZrx6Fwsc455evpMhW17dt1tkwk";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> SolanaClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new(
        "https://api.mainnet-beta.solana.com".to_string(),
        reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build().unwrap(),
    );
    let rpc_client = JsonRpcClient::new(reqwest_client);
    SolanaClient::new(rpc_client)
}

use gem_solana::rpc::client::SolanaClient;
use gem_jsonrpc::JsonRpcClient;
use gem_client::ReqwestClient;

pub fn create_test_client() -> SolanaClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://api.mainnet-beta.solana.com".to_string(), reqwest::Client::new());
    let rpc_client = JsonRpcClient::new(reqwest_client);
    SolanaClient::new(rpc_client)
}
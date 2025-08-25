use gem_client::ReqwestClient;
use gem_jsonrpc::JsonRpcClient;
use gem_solana::rpc::client::SolanaClient;

pub const TEST_ADDRESS: &str = "6sbzC1eH4FTujJXWj51eQe25cYvr4xfXbJ1vAj7j2k5J";

pub fn create_test_client() -> SolanaClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://api.mainnet-beta.solana.com".to_string(), reqwest::Client::new());
    let rpc_client = JsonRpcClient::new(reqwest_client);
    SolanaClient::new(rpc_client)
}

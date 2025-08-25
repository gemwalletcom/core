use gem_aptos::rpc::client::AptosClient;
use gem_client::ReqwestClient;

pub const TEST_ADDRESS: &str = "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc";

pub fn create_aptos_test_client() -> AptosClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://fullnode.mainnet.aptoslabs.com".to_string(), reqwest::Client::new());
    AptosClient::new(reqwest_client)
}

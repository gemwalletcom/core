use gem_algorand::rpc::client::AlgorandClient;
use gem_client::ReqwestClient;

pub const TEST_ADDRESS: &str = "YTMHVGLHWSV72OQF7SGTQZ4BTDNKQVRGM5SYBXUTKWU2RWJKUV7TVKGQPU";

pub fn create_algorand_test_client() -> AlgorandClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://mainnet-algorand.api.purestake.io/ps2".to_string(), reqwest::Client::new());
    AlgorandClient::new(reqwest_client)
}

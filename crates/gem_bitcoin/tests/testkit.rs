use gem_bitcoin::rpc::client::BitcoinClient;
use gem_client::ReqwestClient;
use primitives::BitcoinChain;

pub const TEST_ADDRESS: &str = "bc1qk9cu0nj5czvalnvmlsyc8tmqh8d6f0v9plrrdr";

pub fn create_bitcoin_test_client() -> BitcoinClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://blockbook.btc.zelcore.io".to_string(), reqwest::Client::new());
    BitcoinClient::new(reqwest_client, BitcoinChain::Bitcoin)
}

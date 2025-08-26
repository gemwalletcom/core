use gem_bitcoin::rpc::client::BitcoinClient;
use gem_client::ReqwestClient;
use primitives::BitcoinChain;

pub const TEST_ADDRESS: &str = "bc1qk9cu0nj5czvalnvmlsyc8tmqh8d6f0v9plrrdr";
pub const TEST_TRANSACTION_ID: &str = "654c6a28f7ff1915d2b9abc2e18e32a37e0196203d64aced6221651f003f5e94";

pub fn create_bitcoin_test_client() -> BitcoinClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://blockbook.btc.zelcore.io".to_string(), reqwest::Client::new());
    BitcoinClient::new(reqwest_client, BitcoinChain::Bitcoin)
}

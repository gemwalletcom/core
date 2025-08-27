#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::BitcoinClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use primitives::BitcoinChain;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "bc1qk9cu0nj5czvalnvmlsyc8tmqh8d6f0v9plrrdr";
#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_TRANSACTION_ID: &str = "654c6a28f7ff1915d2b9abc2e18e32a37e0196203d64aced6221651f003f5e94";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_bitcoin_test_client() -> BitcoinClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://blockbook.btc.zelcore.io".to_string(), reqwest::Client::new());
    BitcoinClient::new(reqwest_client, BitcoinChain::Bitcoin)
}

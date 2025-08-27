#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::PolkadotClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_polkadot_test_client() -> PolkadotClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://polkadot-public-sidecar.parity-chains.parity.io".to_string(), reqwest::Client::new());
    PolkadotClient::new(reqwest_client)
}

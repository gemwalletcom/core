use gem_client::ReqwestClient;
use gem_polkadot::rpc::PolkadotClient;

pub const TEST_ADDRESS: &str = "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5";

pub fn create_polkadot_test_client() -> PolkadotClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://polkadot-public-sidecar.parity-chains.parity.io".to_string(), reqwest::Client::new());
    PolkadotClient::new(reqwest_client)
}

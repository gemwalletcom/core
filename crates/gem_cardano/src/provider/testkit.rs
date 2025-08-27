#[cfg(all(test, feature = "integration_tests"))]
use crate::rpc::client::CardanoClient;
#[cfg(all(test, feature = "integration_tests"))]
use gem_client::ReqwestClient;

#[cfg(all(test, feature = "integration_tests"))]
pub const TEST_ADDRESS: &str = "addr1qxf9s6vztx72hukln0r3p795ce6usw5rphsurac22h7f4xt8f32xsvyefel239ly4jev8ump855ynw85q56vh82sxzdsxycpzv";

#[cfg(all(test, feature = "integration_tests"))]
pub fn create_test_client() -> CardanoClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new(
        "https://cardano-mainnet.blockfrost.io/api/v0".to_string(), 
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap()
    );
    CardanoClient::new(reqwest_client)
}
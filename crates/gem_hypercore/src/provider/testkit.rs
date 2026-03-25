#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::{HyperCoreClient, InMemoryPreferences};
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use std::sync::Arc;

#[cfg(all(test, feature = "chain_integration_tests"))]
use primitives::asset_constants::HYPERCORE_SPOT_USDC_TOKEN_ID;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
#[cfg(test)]
pub const TEST_TRANSACTION_ID: &str = "0x9b4d63110c57f2e19cc7042ce90e300202f500f6a75b11b33f160e63cb5bcccc";
#[cfg(test)]
pub const TEST_TRANSACTION_ORDER_ID: &str = "187530505765";
#[cfg(all(test, feature = "chain_integration_tests"))]
pub const USDC_TOKEN_ID: &str = HYPERCORE_SPOT_USDC_TOKEN_ID;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_hypercore_test_client() -> HyperCoreClient<ReqwestClient> {
    let preferences = Arc::new(InMemoryPreferences::new());
    let secure_preferences = Arc::new(InMemoryPreferences::new());

    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.hypercore.url, reqwest::Client::new());
    HyperCoreClient::new_with_preferences(reqwest_client, preferences, secure_preferences)
}

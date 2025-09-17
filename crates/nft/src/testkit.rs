#![cfg(any(test, feature = "nft_integration_tests"))]

pub const TEST_ETHEREUM_ADDRESS: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
pub const TEST_ETHEREUM_CONTRACT_ADDRESS: &str = "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D";
pub const TEST_SOLANA_ADDRESS: &str = "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H";
pub const TEST_SOLANA_COLLECTION: &str = "okay_bears";
pub const TEST_SOLANA_COLLECTION_POOKS: &str = "pooks";
pub const TEST_SOLANA_TOKEN_ID: &str = "HP82kPNXnQcozjDrV4dLYfV6wwABQDMVPJXezDbZXHEy";

#[cfg(feature = "nft_integration_tests")]
use crate::providers::magiceden::client::MagicEdenClient;
#[cfg(feature = "nft_integration_tests")]
use crate::providers::opensea::client::OpenSeaClient;
#[cfg(feature = "nft_integration_tests")]
use settings::Settings;

#[cfg(feature = "nft_integration_tests")]
fn get_test_settings() -> Settings {
    let settings_path = std::env::current_dir().expect("Failed to get current directory").join("../../Settings.yaml");
    Settings::new_setting_path(settings_path).expect("Failed to load settings for tests")
}

#[cfg(feature = "nft_integration_tests")]
pub fn create_opensea_test_client() -> OpenSeaClient {
    let settings = get_test_settings();
    OpenSeaClient::new(&settings.nft.opensea.key.secret)
}

#[cfg(feature = "nft_integration_tests")]
pub fn create_magiceden_test_client() -> MagicEdenClient {
    let settings = get_test_settings();
    MagicEdenClient::new(&settings.nft.magiceden.key.secret)
}

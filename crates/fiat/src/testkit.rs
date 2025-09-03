#[cfg(all(test, feature = "fiat_integration_tests"))]
use crate::client::FiatClient;
#[cfg(all(test, feature = "fiat_integration_tests"))]
use crate::model::FiatMapping;
#[cfg(all(test, feature = "fiat_integration_tests"))]
use crate::providers::{
    banxa::client::BanxaClient, mercuryo::client::MercuryoClient, moonpay::client::MoonPayClient, paybis::client::PaybisClient, transak::client::TransakClient,
};
#[cfg(all(test, feature = "fiat_integration_tests"))]
use settings::Settings;

#[cfg(all(test, feature = "fiat_integration_tests"))]
fn get_test_settings() -> Settings {
    let settings_path = std::env::current_dir().expect("Failed to get current directory").join("../../Settings.yaml");
    Settings::new_setting_path(settings_path).expect("Failed to load settings for tests")
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_transak_test_client() -> TransakClient {
    let settings = get_test_settings();
    let client = FiatClient::request_client(settings.fiat.timeout);
    TransakClient::new(client, settings.transak.key.public, settings.transak.key.secret)
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_moonpay_test_client() -> MoonPayClient {
    let settings = get_test_settings();
    let client = FiatClient::request_client(settings.fiat.timeout);
    MoonPayClient::new(client, settings.moonpay.key.public, settings.moonpay.key.secret)
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_paybis_test_client() -> PaybisClient {
    let settings = get_test_settings();
    let client = FiatClient::request_client(settings.fiat.timeout);
    PaybisClient::new(client, settings.paybis.key.public, settings.paybis.key.secret)
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_banxa_test_client() -> BanxaClient {
    let settings = get_test_settings();
    let client = FiatClient::request_client(settings.fiat.timeout);
    BanxaClient::new(client, settings.banxa.url, settings.banxa.key.public, settings.banxa.key.secret)
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub fn create_mercuryo_test_client() -> MercuryoClient {
    let settings = get_test_settings();
    let client = FiatClient::request_client(settings.fiat.timeout);
    MercuryoClient::new(client, settings.mercuryo.key.public, settings.mercuryo.key.secret, settings.mercuryo.key.token)
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
impl FiatMapping {
    pub fn mock() -> Self {
        FiatMapping {
            symbol: "BTC".to_string(),
            network: Some("BITCOIN".to_string()),
            unsupported_countries: std::collections::HashMap::new(),
        }
    }
}

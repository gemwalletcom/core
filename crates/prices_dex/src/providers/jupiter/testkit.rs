#[cfg(feature = "price_integration_tests")]
use crate::JupiterProvider;

#[cfg(feature = "price_integration_tests")]
pub fn create_jupiter_test_provider() -> JupiterProvider {
    let settings = settings::testkit::get_test_settings();
    JupiterProvider::new(&settings.prices.jupiter.url)
}

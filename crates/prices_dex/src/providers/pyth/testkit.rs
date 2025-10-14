#[cfg(feature = "price_integration_tests")]
use crate::PythProvider;
#[cfg(feature = "price_integration_tests")]
use crate::providers::pyth::client::PythClient;

#[cfg(feature = "price_integration_tests")]
pub fn create_pyth_test_provider() -> PythProvider {
    let settings = settings::testkit::get_test_settings();
    PythProvider {
        pyth_client: PythClient::new(&settings.prices.pyth.url),
    }
}

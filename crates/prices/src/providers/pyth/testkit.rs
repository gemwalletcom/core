#[cfg(feature = "price_integration_tests")]
use crate::PythProvider;
#[cfg(feature = "price_integration_tests")]
use gem_client::ReqwestClient;

#[cfg(feature = "price_integration_tests")]
pub fn create_pyth_test_provider() -> PythProvider {
    let settings = settings::testkit::get_test_settings();
    PythProvider::new(ReqwestClient::new_test_client(settings.prices.pyth.url))
}

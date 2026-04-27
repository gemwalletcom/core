pub mod chart_client;
pub mod markets_client;
pub mod price_alert_client;
pub mod price_client;

use prices::{PriceAssetsProvider, PriceProviderConfig, PriceProviderEndpoints};
use primitives::{ConfigParamKey, PriceProvider};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;

pub use chart_client::ChartClient;
pub use markets_client::MarketsClient;
pub use price_alert_client::{PriceAlertClient, PriceAlertNotification, PriceAlertRules};
pub use price_client::PriceClient;

pub type PriceProviders = HashMap<PriceProvider, Arc<dyn PriceAssetsProvider>>;

pub fn build_price_providers(
    endpoints: &PriceProviderEndpoints,
    providers: impl IntoIterator<Item = PriceProvider>,
    config: &ConfigCacher,
) -> Result<PriceProviders, Box<dyn Error + Send + Sync>> {
    providers
        .into_iter()
        .map(|provider| {
            let provider_config = PriceProviderConfig {
                min_score: config.get_param_f64(&ConfigParamKey::PriceProviderAssetsMinScore(provider))?,
            };
            Ok((provider, endpoints.provider(provider, provider_config)))
        })
        .collect()
}

pub mod chart_client;
pub mod markets_client;
pub mod price_alert_client;
pub mod price_client;

use prices::{PriceAssetsProvider, PriceProviderEndpoints};
use primitives::PriceProvider;
use std::collections::HashMap;
use std::sync::Arc;

pub use chart_client::ChartClient;
pub use markets_client::MarketsClient;
pub use price_alert_client::{PriceAlertClient, PriceAlertNotification, PriceAlertRules};
pub use price_client::PriceClient;

pub type PriceProviders = HashMap<PriceProvider, Arc<dyn PriceAssetsProvider>>;

pub fn build_price_providers(endpoints: &PriceProviderEndpoints, providers: impl IntoIterator<Item = PriceProvider>) -> PriceProviders {
    providers.into_iter().map(|provider| (provider, endpoints.provider(provider))).collect()
}

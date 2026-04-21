use serde::{Deserialize, Serialize};

use crate::PriceProvider;

/// The resolved (provider, provider_price_id) pair used to key prices, charts and any other
/// provider-scoped data. `id()` produces the synthetic `prices.id` used across the schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetPriceKey {
    pub provider: PriceProvider,
    pub provider_price_id: String,
}

impl AssetPriceKey {
    pub fn new(provider: PriceProvider, provider_price_id: String) -> Self {
        Self { provider, provider_price_id }
    }

    pub fn id(&self) -> String {
        self.provider.price_id(&self.provider_price_id)
    }
}

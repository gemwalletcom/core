use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::CacheKey;
use cacher::CacherClient;
use primitives::Chain;
use settings_chain::ChainProviders;
use storage::{Database, WalletsRepository};

pub struct PerpetualAddressRefresher {
    providers: Arc<ChainProviders>,
    database: Database,
    cacher: CacherClient,
}

impl PerpetualAddressRefresher {
    pub fn new(providers: Arc<ChainProviders>, database: Database, cacher: CacherClient) -> Self {
        Self { providers, database, cacher }
    }

    pub async fn update(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let referred_addresses = self.providers.get_perpetual_referred_addresses(chain).await?;
        if referred_addresses.is_empty() {
            return Ok(0);
        }

        let active_addresses: Vec<String> = self
            .database
            .wallets()?
            .get_subscriptions_by_chain_addresses(chain, referred_addresses)?
            .into_iter()
            .map(|s| s.address)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        if active_addresses.is_empty() {
            return Ok(0);
        }

        self.cacher.add_to_set_cached(CacheKey::PerpetualActiveAddresses(chain.as_ref()), &active_addresses).await
    }
}

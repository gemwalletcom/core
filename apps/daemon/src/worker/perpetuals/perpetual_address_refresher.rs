use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::{CacheKey, CacherClient};
use gem_tracing::info_with_fields;
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
        let referred_count = referred_addresses.len();
        let key = CacheKey::PerpetualTrackedAddresses(chain.as_ref());

        let tracked_addresses: Vec<String> = if referred_addresses.is_empty() {
            vec![]
        } else {
            self.database
                .wallets()?
                .get_subscriptions_by_chain_addresses(chain, referred_addresses)?
                .into_iter()
                .map(|s| s.address)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        };

        self.cacher.set_cached(key, &tracked_addresses).await?;

        info_with_fields!("perpetual_refresher", chain = chain.as_ref(), referred = referred_count, tracked = tracked_addresses.len());

        Ok(tracked_addresses.len())
    }
}

use std::error::Error;
use std::sync::Arc;

use cacher::{CacheKey, CacherClient};
use primitives::CrossChainProvider;
use swapper::swapper::GemSwapper;

pub struct VaultAddressesUpdater {
    swapper: Arc<GemSwapper>,
    cacher: CacherClient,
}

impl VaultAddressesUpdater {
    pub fn new(swapper: Arc<GemSwapper>, cacher: CacherClient) -> Self {
        Self { swapper, cacher }
    }

    pub async fn update(&self, provider: CrossChainProvider, from_timestamp: Option<u64>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let addresses = self.swapper.get_vault_addresses(&provider.into(), from_timestamp).await?;
        self.cacher.add_to_set_cached(CacheKey::SwapVaultAddresses(provider.as_ref()), &addresses).await
    }
}

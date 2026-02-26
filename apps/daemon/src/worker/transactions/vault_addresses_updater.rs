use std::error::Error;
use std::sync::Arc;

use cacher::{CacheKey, CacherClient};
use primitives::SwapProvider;
use swapper::swapper::GemSwapper;

pub struct VaultAddressesUpdater {
    swapper: Arc<GemSwapper>,
    cacher: CacherClient,
}

impl VaultAddressesUpdater {
    pub fn new(swapper: Arc<GemSwapper>, cacher: CacherClient) -> Self {
        Self { swapper, cacher }
    }

    pub async fn update(&self, provider: SwapProvider) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let addresses = self.swapper.get_vault_addresses(&provider).await?;
        let count = addresses.len();
        self.cacher.set_cached(CacheKey::SwapVaultAddresses(provider.as_ref()), &addresses).await?;
        Ok(count)
    }
}

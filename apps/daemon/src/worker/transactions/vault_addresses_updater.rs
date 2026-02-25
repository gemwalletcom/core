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
        let addresses: Vec<String> = self.swapper.get_vault_addresses(&provider).await?.into_iter().map(|a| a.to_lowercase()).collect();
        let count = addresses.len();
        let hash_key = CacheKey::SwapVaultAddresses.key();
        self.cacher.set_hset(&hash_key, provider.as_ref(), &addresses).await?;
        Ok(count)
    }
}

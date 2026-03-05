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

    pub async fn update(&self, provider: SwapProvider, from_timestamp: Option<u64>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let vault_addresses = self.swapper.get_vault_addresses(&provider, from_timestamp).await?;
        let deposit_count = self
            .cacher
            .add_to_set_cached(CacheKey::SwapDepositAddresses(provider.as_ref()), &vault_addresses.deposit)
            .await?;
        let send_count = self.cacher.add_to_set_cached(CacheKey::SwapSendAddresses(provider.as_ref()), &vault_addresses.send).await?;
        Ok(deposit_count + send_count)
    }
}

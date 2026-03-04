use std::error::Error;

use cacher::{CacheKey, CacherClient};
use primitives::SwapProvider;
use swapper::cross_chain::VaultAddressMap;

#[derive(Clone)]
pub struct SwapVaultAddressClient {
    cacher: CacherClient,
}

impl SwapVaultAddressClient {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub async fn get_address_map(&self) -> Result<VaultAddressMap, Box<dyn Error + Send + Sync>> {
        let providers = SwapProvider::cross_chain_providers();
        let keys: Vec<String> = providers.iter().map(|p| CacheKey::SwapVaultAddresses(p.as_ref()).key()).collect();
        let results = self.cacher.get_set_members_grouped(keys).await?;
        Ok(providers
            .into_iter()
            .zip(results)
            .flat_map(|(provider, members)| members.into_iter().map(move |addr| (addr, provider)))
            .collect())
    }
}

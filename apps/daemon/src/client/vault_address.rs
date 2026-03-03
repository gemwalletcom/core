use std::error::Error;

use cacher::{CacheKey, CacherClient};
use swapper::cross_chain::{self, VaultAddressMap};

#[derive(Clone)]
pub struct SwapVaultAddressClient {
    cacher: CacherClient,
}

impl SwapVaultAddressClient {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub async fn get_address_map(&self) -> Result<VaultAddressMap, Box<dyn Error + Send + Sync>> {
        let mut map = VaultAddressMap::new();
        for provider in cross_chain::providers() {
            let key = CacheKey::SwapVaultAddresses(provider.as_ref()).key();
            let members = self.cacher.get_set_members_cached(vec![key]).await?;
            let swap_provider = provider.into();
            for addr in members {
                map.insert(addr, swap_provider);
            }
        }
        Ok(map)
    }
}

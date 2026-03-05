use std::error::Error;

use cacher::{CacheKey, CacherClient};
use primitives::SwapProvider;
use std::collections::HashMap;

use swapper::SwapperProvider;

type AddressMap = HashMap<String, SwapperProvider>;

#[derive(Clone)]
pub struct SwapVaultAddressClient {
    cacher: CacherClient,
}

impl SwapVaultAddressClient {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub async fn get_deposit_address_map(&self) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        self.get_address_map(|p| CacheKey::SwapDepositAddresses(p)).await
    }

    pub async fn get_send_address_map(&self) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        self.get_address_map(|p| CacheKey::SwapSendAddresses(p)).await
    }

    async fn get_address_map<F>(&self, key_fn: F) -> Result<AddressMap, Box<dyn Error + Send + Sync>>
    where
        F: Fn(&str) -> CacheKey<'_>,
    {
        let providers = SwapProvider::cross_chain_providers();
        let keys: Vec<String> = providers.iter().map(|p| key_fn(p.as_ref()).key()).collect();
        let results = self.cacher.get_set_members_grouped(keys).await?;
        Ok(providers
            .into_iter()
            .zip(results)
            .flat_map(|(provider, members)| members.into_iter().map(move |addr| (addr, provider)))
            .collect())
    }
}

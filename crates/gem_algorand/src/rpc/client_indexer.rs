use std::error::Error;

use crate::models::{Account, AssetResponse, Block, BlockHeaders, Transactions};

#[cfg(feature = "rpc")]
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider, ChainStaking, ChainTraits, ChainTransactionLoad};
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use primitives::Chain;

#[derive(Debug)]
pub struct AlgorandClientIndexer<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> AlgorandClientIndexer<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            chain: Chain::Algorand,
        }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_block_headers(&self, limit: u64) -> Result<BlockHeaders, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/block-headers?limit={}", limit)).await?)
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/accounts/{}", address)).await?)
    }

    pub async fn get_asset(&self, asset_id: &str) -> Result<AssetResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/assets/{}", asset_id)).await?)
    }

    pub async fn get_account_transactions(&self, address: &str) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/accounts/{}/transactions", address)).await?)
    }

    pub async fn get_block(&self, block_number: u64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/blocks/{}", block_number)).await?)
    }
}

#[cfg(feature = "rpc")]
impl<C: Client> ChainProvider for AlgorandClientIndexer<C> {
    fn get_chain(&self) -> Chain {
        self.chain
    }
}

#[cfg(feature = "rpc")]
impl<C: Client> ChainStaking for AlgorandClientIndexer<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainAccount for AlgorandClientIndexer<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainPerpetual for AlgorandClientIndexer<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainAddressStatus for AlgorandClientIndexer<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainTransactionLoad for AlgorandClientIndexer<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainTraits for AlgorandClientIndexer<C> {}

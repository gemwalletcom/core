use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::FeePriorityValue;

use super::state_mapper;
use crate::rpc::client::NearClient;

#[async_trait]
impl<C: Client> ChainState for NearClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_near_genesis_config().await?.chain_id)
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_near_latest_block().await?.header.height)
    }

    async fn get_fees(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_near_gas_price().await?;
        state_mapper::map_gas_price_to_priorities(&gas_price)
    }
}

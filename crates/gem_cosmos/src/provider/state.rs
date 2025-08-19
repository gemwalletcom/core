use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::FeePriorityValue;

use crate::rpc::client::CosmosClient;

#[async_trait]
impl<C: Client> ChainState for CosmosClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        use crate::models::block::CosmosNodeInfoResponse;
        
        let node_info: CosmosNodeInfoResponse = self.client.get("/cosmos/base/tendermint/v1beta1/node_info").await?;
        Ok(node_info.default_node_info.network)
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let block = self.get_block("latest").await?;
        Ok(block.block.header.height.parse()?)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        let base_fee = self.get_base_fee();
        let cosmos_chain = self.get_chain();
        
        Ok(crate::provider::state_mapper::calculate_fee_rates(cosmos_chain, base_fee.into()))
    }
}
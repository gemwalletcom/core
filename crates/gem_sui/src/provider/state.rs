#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainState;
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use primitives::FeePriorityValue;
#[cfg(feature = "rpc")]
use std::error::Error;

#[cfg(feature = "rpc")]
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client> ChainState for SuiClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.get_chain_id().await
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        self.get_latest_block().await
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        Ok(vec![FeePriorityValue {
            priority: primitives::FeePriority::Normal,
            value: gas_price.to_string(),
        }])
    }
}
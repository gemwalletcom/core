use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::FeePriorityValue;

use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainState for SolanaClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.get_genesis_hash().await
    }
    
    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        self.get_slot().await
    }
    
    async fn get_fee_rates(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        let prioritization_fees = self.get_recent_prioritization_fees().await?;
        let input_type = primitives::TransactionInputType::Transfer(primitives::Asset {
            id: primitives::AssetId::from_chain(self.get_chain()),
            chain: self.get_chain(),
            token_id: None,
            name: "Solana".to_string(),
            symbol: "SOL".to_string(),
            decimals: 9,
            asset_type: primitives::AssetType::NATIVE,
        });
        
        Ok(crate::provider::preload_mapper::calculate_fee_rates(&input_type, &prioritization_fees))
    }
}
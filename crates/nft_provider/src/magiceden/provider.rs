use std::error::Error;

use primitives::NFTAsset;
use primitives::NFTAssetId;
use primitives::NFTCollectionId;
use primitives::{Chain, NFTCollection};

use crate::MagicEdenClient;
use crate::NFTProvider;

#[async_trait::async_trait]
impl NFTProvider for MagicEdenClient {
    fn name(&self) -> &'static str {
        "MagicEden"
    }

    fn get_chains(&self) -> Vec<Chain> {
        vec![Chain::Solana]
    }

    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_nfts_by_account(&address)
            .await?
            .into_iter()
            .flat_map(|x| x.as_asset_id(chain))
            .collect())
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        Ok(self.get_collection_id(&collection_id.contract_address).await?.as_primitive(collection_id))
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let owner = self.get_token_owner(&asset_id.token_id).await?;
        let asset = self
            .get_asset_id(&asset_id.token_id)
            .await?
            .as_primitive(asset_id, owner)
            .ok_or("Asset not found")?;
        Ok(asset)
    }
}

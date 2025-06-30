use std::error::Error;

use primitives::NFTAsset;
use primitives::NFTAssetId;
use primitives::NFTCollectionId;
use primitives::{Chain, NFTCollection};

use crate::NFTProvider;
use crate::OpenSeaClient;

#[async_trait::async_trait]
impl NFTProvider for OpenSeaClient {
    fn name(&self) -> &'static str {
        "OpenSea"
    }

    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_nfts_by_account(chain.as_ref(), &address)
            .await?
            .nfts
            .into_iter()
            .flat_map(|x| x.as_asset_id(chain))
            .collect())
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_collection_id(collection_id.chain.as_ref(), &collection_id.contract_address)
            .await?
            .as_primitive(collection_id))
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let asset = self
            .get_asset_id(asset_id.chain.as_ref(), &asset_id.contract_address, &asset_id.token_id)
            .await?
            .nft
            .as_primitive(asset_id)
            .ok_or("Asset not found")?;
        Ok(asset)
    }
}

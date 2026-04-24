use std::error::Error;

use gem_client::Client;
use gem_ton::rpc::client::TonClient;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId, NFTData};

use super::mapper::{map_asset, map_asset_ids, map_collection, map_nft_data};
use crate::provider::NFTProvider;

#[async_trait::async_trait]
impl<C: Client + Send + Sync> NFTProvider for TonClient<C> {
    fn name(&self) -> &'static str {
        "Ton"
    }

    fn chains(&self) -> &'static [NFTChain] {
        &[NFTChain::Ton]
    }

    async fn get_assets(&self, _chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        let response = self.get_nft_items_by_owner(&address).await?;
        Ok(map_asset_ids(&response))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let response = self.get_nft_collection(&collection_id.contract_address).await?;
        map_collection(response, collection_id).ok_or_else(|| "Collection not found".into())
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let response = self.get_nft_item(&asset_id.token_id).await?;
        map_asset(response, asset_id).ok_or_else(|| "Asset not found".into())
    }

    async fn get_nft_data(&self, _chain: Chain, address: String) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let response = self.get_nft_items_by_owner(&address).await?;
        Ok(map_nft_data(response))
    }
}

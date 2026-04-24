use std::error::Error;

use gem_client::Client;
use gem_ton::rpc::client::TonClient;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId};

use super::mapper::{map_asset, map_asset_ids, map_collection, map_nft_assets};
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
        let items = self.get_nft_items_by_owner(&address).await?;
        Ok(map_asset_ids(&items))
    }

    async fn get_nft_assets(&self, _chain: Chain, address: String) -> Result<Vec<NFTAsset>, Box<dyn Error + Send + Sync>> {
        let items = self.get_nft_items_by_owner(&address).await?;
        Ok(map_nft_assets(items))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let collection = self.get_nft_collection(&collection_id.contract_address).await?;
        Ok(map_collection(collection, collection_id))
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let item = self.get_nft_item(&asset_id.token_id).await?;
        map_asset(item, asset_id).ok_or_else(|| "Asset not found".into())
    }
}

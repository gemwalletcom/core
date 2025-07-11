use std::error::Error;

use crate::database::nft::NftStore;
use crate::models::{nft_asset::UpdateNftAssetImageUrl, nft_collection::UpdateNftCollectionImageUrl, nft_link::NftLink, NftAsset, NftCollection, NftType};
use crate::DatabaseClient;

pub trait NftRepository {
    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAsset>, Box<dyn Error + Send + Sync>>;
    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAsset, Box<dyn Error + Send + Sync>>;
    fn add_nft_assets(&mut self, values: Vec<NftAsset>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrl) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollection>, Box<dyn Error + Send + Sync>>;
    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollection, Box<dyn Error + Send + Sync>>;
    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollection>, Box<dyn Error + Send + Sync>>;
    fn get_nft_collection_links(&mut self, collection_id: &str) -> Result<Vec<NftLink>, Box<dyn Error + Send + Sync>>;
    fn add_nft_collections(&mut self, values: Vec<NftCollection>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrl) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn add_nft_types(&mut self, values: Vec<NftType>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn add_nft_collections_links(&mut self, values: Vec<NftLink>) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl NftRepository for DatabaseClient {
    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAsset>, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::get_nft_assets(self, asset_ids)?)
    }

    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAsset, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::get_nft_asset(self, asset_id)?)
    }

    fn add_nft_assets(&mut self, values: Vec<NftAsset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::add_nft_assets(self, values)?)
    }

    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrl) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::update_nft_asset_image_url(self, update)?)
    }

    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollection>, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::get_nft_collections_all(self)?)
    }

    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollection, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::get_nft_collection(self, collection_id)?)
    }

    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollection>, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::get_nft_collections(self, ids)?)
    }

    fn get_nft_collection_links(&mut self, collection_id: &str) -> Result<Vec<NftLink>, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::get_nft_collection_links(self, collection_id)?)
    }

    fn add_nft_collections(&mut self, values: Vec<NftCollection>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::add_nft_collections(self, values)?)
    }

    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrl) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::update_nft_collection_image_url(self, update)?)
    }

    fn add_nft_types(&mut self, values: Vec<NftType>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::add_nft_types(self, values)?)
    }

    fn add_nft_collections_links(&mut self, values: Vec<NftLink>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(NftStore::add_nft_collections_links(self, values)?)
    }
}

use crate::DatabaseError;

use crate::DatabaseClient;
use crate::database::nft::NftStore;
use crate::models::{NftAsset, NftCollection, NftType, nft_asset::UpdateNftAssetImageUrl, nft_collection::UpdateNftCollectionImageUrl, nft_link::NftLink};

pub trait NftRepository {
    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAsset>, DatabaseError>;
    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAsset, DatabaseError>;
    fn add_nft_assets(&mut self, values: Vec<NftAsset>) -> Result<usize, DatabaseError>;
    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrl) -> Result<usize, DatabaseError>;
    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollection>, DatabaseError>;
    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollection, DatabaseError>;
    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollection>, DatabaseError>;
    fn get_nft_collection_links(&mut self, collection_id: &str) -> Result<Vec<NftLink>, DatabaseError>;
    fn add_nft_collections(&mut self, values: Vec<NftCollection>) -> Result<usize, DatabaseError>;
    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrl) -> Result<usize, DatabaseError>;
    fn add_nft_types(&mut self, values: Vec<NftType>) -> Result<usize, DatabaseError>;
    fn add_nft_collections_links(&mut self, values: Vec<NftLink>) -> Result<usize, DatabaseError>;
}

impl NftRepository for DatabaseClient {
    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAsset>, DatabaseError> {
        Ok(NftStore::get_nft_assets(self, asset_ids)?)
    }

    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAsset, DatabaseError> {
        Ok(NftStore::get_nft_asset(self, asset_id)?)
    }

    fn add_nft_assets(&mut self, values: Vec<NftAsset>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_assets(self, values)?)
    }

    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrl) -> Result<usize, DatabaseError> {
        Ok(NftStore::update_nft_asset_image_url(self, update)?)
    }

    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollection>, DatabaseError> {
        Ok(NftStore::get_nft_collections_all(self)?)
    }

    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollection, DatabaseError> {
        Ok(NftStore::get_nft_collection(self, collection_id)?)
    }

    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollection>, DatabaseError> {
        Ok(NftStore::get_nft_collections(self, ids)?)
    }

    fn get_nft_collection_links(&mut self, collection_id: &str) -> Result<Vec<NftLink>, DatabaseError> {
        Ok(NftStore::get_nft_collection_links(self, collection_id)?)
    }

    fn add_nft_collections(&mut self, values: Vec<NftCollection>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_collections(self, values)?)
    }

    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrl) -> Result<usize, DatabaseError> {
        Ok(NftStore::update_nft_collection_image_url(self, update)?)
    }

    fn add_nft_types(&mut self, values: Vec<NftType>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_types(self, values)?)
    }

    fn add_nft_collections_links(&mut self, values: Vec<NftLink>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_collections_links(self, values)?)
    }
}

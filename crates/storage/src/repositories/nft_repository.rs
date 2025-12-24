use crate::DatabaseError;

use crate::DatabaseClient;
use crate::database::nft::NftStore;
use crate::models::{
    NftAssetRow, NftCollectionRow, NftTypeRow, nft_asset::UpdateNftAssetImageUrlRow, nft_collection::UpdateNftCollectionImageUrlRow, nft_link::NftLinkRow,
    nft_report::NewNftReportRow,
};

pub trait NftRepository {
    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAssetRow>, DatabaseError>;
    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAssetRow, DatabaseError>;
    fn add_nft_assets(&mut self, values: Vec<NftAssetRow>) -> Result<usize, DatabaseError>;
    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrlRow) -> Result<usize, DatabaseError>;
    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollectionRow>, DatabaseError>;
    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollectionRow, DatabaseError>;
    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollectionRow>, DatabaseError>;
    fn get_nft_collection_links(&mut self, collection_id: &str) -> Result<Vec<NftLinkRow>, DatabaseError>;
    fn add_nft_collections(&mut self, values: Vec<NftCollectionRow>) -> Result<usize, DatabaseError>;
    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrlRow) -> Result<usize, DatabaseError>;
    fn add_nft_types(&mut self, values: Vec<NftTypeRow>) -> Result<usize, DatabaseError>;
    fn add_nft_collections_links(&mut self, values: Vec<NftLinkRow>) -> Result<usize, DatabaseError>;
    fn add_nft_report(&mut self, report: NewNftReportRow) -> Result<usize, DatabaseError>;
}

impl NftRepository for DatabaseClient {
    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAssetRow>, DatabaseError> {
        Ok(NftStore::get_nft_assets(self, asset_ids)?)
    }

    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAssetRow, DatabaseError> {
        Ok(NftStore::get_nft_asset(self, asset_id)?)
    }

    fn add_nft_assets(&mut self, values: Vec<NftAssetRow>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_assets(self, values)?)
    }

    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrlRow) -> Result<usize, DatabaseError> {
        Ok(NftStore::update_nft_asset_image_url(self, update)?)
    }

    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollectionRow>, DatabaseError> {
        Ok(NftStore::get_nft_collections_all(self)?)
    }

    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollectionRow, DatabaseError> {
        Ok(NftStore::get_nft_collection(self, collection_id)?)
    }

    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollectionRow>, DatabaseError> {
        Ok(NftStore::get_nft_collections(self, ids)?)
    }

    fn get_nft_collection_links(&mut self, collection_id: &str) -> Result<Vec<NftLinkRow>, DatabaseError> {
        Ok(NftStore::get_nft_collection_links(self, collection_id)?)
    }

    fn add_nft_collections(&mut self, values: Vec<NftCollectionRow>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_collections(self, values)?)
    }

    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrlRow) -> Result<usize, DatabaseError> {
        Ok(NftStore::update_nft_collection_image_url(self, update)?)
    }

    fn add_nft_types(&mut self, values: Vec<NftTypeRow>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_types(self, values)?)
    }

    fn add_nft_collections_links(&mut self, values: Vec<NftLinkRow>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_collections_links(self, values)?)
    }

    fn add_nft_report(&mut self, report: NewNftReportRow) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_report(self, report)?)
    }
}

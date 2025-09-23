use crate::DatabaseError;

use crate::DatabaseClient;
use crate::database::tag::TagStore;
use crate::models::{AssetTag, Tag};

pub trait TagRepository {
    fn add_tags(&mut self, values: Vec<Tag>) -> Result<usize, DatabaseError>;
    fn add_assets_tags(&mut self, values: Vec<AssetTag>) -> Result<usize, DatabaseError>;
    fn get_assets_tags(&mut self) -> Result<Vec<AssetTag>, DatabaseError>;
    fn get_assets_tags_for_tag(&mut self, _tag_id: &str) -> Result<Vec<AssetTag>, DatabaseError>;
    fn delete_assets_tags(&mut self, _tag_id: &str) -> Result<usize, DatabaseError>;
    fn set_assets_tags_for_tag(&mut self, _tag_id: &str, asset_ids: Vec<String>) -> Result<usize, DatabaseError>;
    fn get_assets_tags_for_asset(&mut self, _asset_id: &str) -> Result<Vec<AssetTag>, DatabaseError>;
}

impl TagRepository for DatabaseClient {
    fn add_tags(&mut self, values: Vec<Tag>) -> Result<usize, DatabaseError> {
        Ok(TagStore::add_tags(self, values)?)
    }

    fn add_assets_tags(&mut self, values: Vec<AssetTag>) -> Result<usize, DatabaseError> {
        Ok(TagStore::add_assets_tags(self, values)?)
    }

    fn get_assets_tags(&mut self) -> Result<Vec<AssetTag>, DatabaseError> {
        Ok(TagStore::get_assets_tags(self)?)
    }

    fn get_assets_tags_for_tag(&mut self, _tag_id: &str) -> Result<Vec<AssetTag>, DatabaseError> {
        Ok(TagStore::get_assets_tags_for_tag(self, _tag_id)?)
    }

    fn delete_assets_tags(&mut self, _tag_id: &str) -> Result<usize, DatabaseError> {
        Ok(TagStore::delete_assets_tags(self, _tag_id)?)
    }

    fn set_assets_tags_for_tag(&mut self, _tag_id: &str, asset_ids: Vec<String>) -> Result<usize, DatabaseError> {
        Ok(TagStore::set_assets_tags_for_tag(self, _tag_id, asset_ids)?)
    }

    fn get_assets_tags_for_asset(&mut self, _asset_id: &str) -> Result<Vec<AssetTag>, DatabaseError> {
        Ok(TagStore::get_assets_tags_for_asset(self, _asset_id)?)
    }
}

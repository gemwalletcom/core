use std::error::Error;

use crate::database::tag::TagStore;
use crate::models::{AssetTag, Tag};
use crate::DatabaseClient;

pub trait TagRepository {
    fn add_tags(&mut self, values: Vec<Tag>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn add_assets_tags(&mut self, values: Vec<AssetTag>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_assets_tags(&mut self) -> Result<Vec<AssetTag>, Box<dyn Error + Send + Sync>>;
    fn get_assets_tags_for_tag(&mut self, _tag_id: &str) -> Result<Vec<AssetTag>, Box<dyn Error + Send + Sync>>;
    fn delete_assets_tags(&mut self, _tag_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn set_assets_tags_for_tag(&mut self, _tag_id: &str, asset_ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_assets_tags_for_asset(&mut self, _asset_id: &str) -> Result<Vec<AssetTag>, Box<dyn Error + Send + Sync>>;
}

impl TagRepository for DatabaseClient {
    fn add_tags(&mut self, values: Vec<Tag>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(TagStore::add_tags(self, values)?)
    }

    fn add_assets_tags(&mut self, values: Vec<AssetTag>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(TagStore::add_assets_tags(self, values)?)
    }

    fn get_assets_tags(&mut self) -> Result<Vec<AssetTag>, Box<dyn Error + Send + Sync>> {
        Ok(TagStore::get_assets_tags(self)?)
    }

    fn get_assets_tags_for_tag(&mut self, _tag_id: &str) -> Result<Vec<AssetTag>, Box<dyn Error + Send + Sync>> {
        Ok(TagStore::get_assets_tags_for_tag(self, _tag_id)?)
    }

    fn delete_assets_tags(&mut self, _tag_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(TagStore::delete_assets_tags(self, _tag_id)?)
    }

    fn set_assets_tags_for_tag(&mut self, _tag_id: &str, asset_ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(TagStore::set_assets_tags_for_tag(self, _tag_id, asset_ids)?)
    }

    fn get_assets_tags_for_asset(&mut self, _asset_id: &str) -> Result<Vec<AssetTag>, Box<dyn Error + Send + Sync>> {
        Ok(TagStore::get_assets_tags_for_asset(self, _asset_id)?)
    }
}

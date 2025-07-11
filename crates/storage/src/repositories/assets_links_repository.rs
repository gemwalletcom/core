use std::error::Error;

use crate::database::assets_links::AssetsLinksStore;
use crate::{models::AssetLink, DatabaseClient};
use primitives::AssetLink as PrimitiveAssetLink;

pub trait AssetsLinksRepository {
    fn add_assets_links(&mut self, asset_id: &str, values: Vec<PrimitiveAssetLink>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_asset_links(&mut self, asset_id: &str) -> Result<Vec<PrimitiveAssetLink>, Box<dyn Error + Send + Sync>>;
}

impl AssetsLinksRepository for DatabaseClient {
    fn add_assets_links(&mut self, asset_id: &str, values: Vec<PrimitiveAssetLink>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsLinksStore::add_assets_links(
            self,
            values.into_iter().map(|x| AssetLink::from_primitive(asset_id, x)).collect(),
        )?)
    }

    fn get_asset_links(&mut self, asset_id: &str) -> Result<Vec<PrimitiveAssetLink>, Box<dyn Error + Send + Sync>> {
        Ok(AssetsLinksStore::get_asset_links(self, asset_id)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect())
    }
}

use crate::DatabaseError;

use crate::database::assets_links::AssetsLinksStore;
use crate::{DatabaseClient, models::AssetLinkRow};
use primitives::{AssetId, AssetLink as PrimitiveAssetLink};

pub trait AssetsLinksRepository {
    fn add_assets_links(&mut self, asset_id: &AssetId, values: Vec<PrimitiveAssetLink>) -> Result<usize, DatabaseError>;
    fn get_asset_links(&mut self, asset_id: &AssetId) -> Result<Vec<PrimitiveAssetLink>, DatabaseError>;
}

impl AssetsLinksRepository for DatabaseClient {
    fn add_assets_links(&mut self, asset_id: &AssetId, values: Vec<PrimitiveAssetLink>) -> Result<usize, DatabaseError> {
        Ok(AssetsLinksStore::add_assets_links(
            self,
            values.into_iter().map(|x| AssetLinkRow::from_primitive(asset_id, x)).collect(),
        )?)
    }

    fn get_asset_links(&mut self, asset_id: &AssetId) -> Result<Vec<PrimitiveAssetLink>, DatabaseError> {
        Ok(AssetsLinksStore::get_asset_links(self, &asset_id.to_string())?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect())
    }
}

use std::error::Error;

use crate::database::assets::AssetsStore;
use crate::database::assets::{AssetFilter, AssetUpdate};
use crate::{models::Asset, DatabaseClient};
use primitives::{Asset as PrimitiveAsset, AssetBasic};

pub trait AssetsRepository {
    fn get_assets_all(&mut self) -> Result<Vec<PrimitiveAsset>, Box<dyn Error + Send + Sync>>;
    fn add_assets(&mut self, values: Vec<AssetBasic>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_assets(&mut self, asset_ids: Vec<String>, updates: Vec<AssetUpdate>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_assets_bulk(&mut self, asset_ids: Vec<String>, update: AssetUpdate) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn update_asset(&mut self, asset_id: String, update: AssetUpdate) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn upsert_assets(&mut self, values: Vec<PrimitiveAsset>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>>;
    fn get_asset(&mut self, asset_id: &str) -> Result<PrimitiveAsset, Box<dyn Error + Send + Sync>>;
    fn get_asset_full(&mut self, asset_id: &str) -> Result<primitives::AssetFull, Box<dyn Error + Send + Sync>>;
    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<PrimitiveAsset>, Box<dyn Error + Send + Sync>>;
    fn get_assets_basic(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>>;
    fn get_swap_assets(&mut self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
    fn get_swap_assets_version(&mut self) -> Result<i32, Box<dyn Error + Send + Sync>>;
    fn add_chains(&mut self, values: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl AssetsRepository for DatabaseClient {
    fn get_assets_all(&mut self) -> Result<Vec<PrimitiveAsset>, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::get_assets_all(self)?.into_iter().map(|x| x.as_primitive()).collect())
    }

    fn add_assets(&mut self, values: Vec<AssetBasic>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::add_assets(
            self,
            values.into_iter().map(|x| Asset::from_primitive(x.asset, x.score, x.properties)).collect(),
        )?)
    }

    fn update_assets(&mut self, asset_ids: Vec<String>, updates: Vec<AssetUpdate>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::update_assets(self, asset_ids, updates)?)
    }

    fn update_assets_bulk(&mut self, asset_ids: Vec<String>, update: AssetUpdate) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::update_assets_bulk(self, asset_ids, update)?)
    }

    fn update_asset(&mut self, asset_id: String, update: AssetUpdate) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::update_asset(self, asset_id, update)?)
    }

    fn upsert_assets(&mut self, values: Vec<PrimitiveAsset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::upsert_assets(
            self,
            values.into_iter().map(Asset::from_primitive_default).collect(),
        )?)
    }

    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::get_assets_by_filter(self, filters)?
            .into_iter()
            .map(|x| x.as_basic_primitive())
            .collect())
    }

    fn get_asset(&mut self, asset_id: &str) -> Result<PrimitiveAsset, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::get_asset(self, asset_id)?.as_primitive())
    }

    fn get_asset_full(&mut self, asset_id: &str) -> Result<primitives::AssetFull, Box<dyn Error + Send + Sync>> {
        use crate::database::assets_links::AssetsLinksStore;
        use crate::database::tag::TagStore;

        let asset = AssetsStore::get_asset(self, asset_id)?;
        let links = AssetsLinksStore::get_asset_links(self, asset_id)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect();
        let tags = TagStore::get_assets_tags_for_asset(self, asset_id)?.into_iter().map(|x| x.tag_id).collect();

        Ok(primitives::AssetFull {
            asset: asset.as_primitive(),
            properties: asset.as_property_primitive(),
            score: asset.as_score_primitive(),
            links,
            tags,
        })
    }

    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<PrimitiveAsset>, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::get_assets(self, asset_ids)?.into_iter().map(|x| x.as_primitive()).collect())
    }

    fn get_assets_basic(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::get_assets(self, asset_ids)?.into_iter().map(|x| x.as_basic_primitive()).collect())
    }

    fn get_swap_assets(&mut self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::get_swap_assets(self)?)
    }

    fn get_swap_assets_version(&mut self) -> Result<i32, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::get_swap_assets_version(self)?)
    }

    fn add_chains(&mut self, values: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsStore::add_chains(self, values)?)
    }
}

use crate::DatabaseError;

use crate::database::assets::AssetsStore;
use crate::database::assets::{AssetFilter, AssetUpdate};
use crate::{DatabaseClient, models::Asset};
use primitives::{Asset as PrimitiveAsset, AssetBasic};

pub trait AssetsRepository {
    fn get_assets_all(&mut self) -> Result<Vec<AssetBasic>, DatabaseError>;
    fn add_assets(&mut self, values: Vec<AssetBasic>) -> Result<usize, DatabaseError>;
    fn update_assets(&mut self, asset_ids: Vec<String>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError>;
    fn update_asset(&mut self, asset_id: String, update: AssetUpdate) -> Result<usize, DatabaseError>;
    fn upsert_assets(&mut self, values: Vec<PrimitiveAsset>) -> Result<usize, DatabaseError>;
    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetBasic>, DatabaseError>;
    fn get_asset(&mut self, asset_id: &str) -> Result<PrimitiveAsset, DatabaseError>;
    fn get_asset_full(&mut self, asset_id: &str) -> Result<primitives::AssetFull, DatabaseError>;
    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<PrimitiveAsset>, DatabaseError>;
    fn get_assets_basic(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetBasic>, DatabaseError>;
    fn get_swap_assets(&mut self) -> Result<Vec<String>, DatabaseError>;
    fn get_swap_assets_version(&mut self) -> Result<i32, DatabaseError>;
    fn add_chains(&mut self, values: Vec<String>) -> Result<usize, DatabaseError>;
}

impl AssetsRepository for DatabaseClient {
    fn get_assets_all(&mut self) -> Result<Vec<AssetBasic>, DatabaseError> {
        Ok(AssetsStore::get_assets_all(self)?.into_iter().map(|x| x.as_basic_primitive()).collect())
    }

    fn add_assets(&mut self, values: Vec<AssetBasic>) -> Result<usize, DatabaseError> {
        Ok(AssetsStore::add_assets(
            self,
            values.into_iter().map(|x| Asset::from_primitive(x.asset, x.score, x.properties)).collect(),
        )?)
    }

    fn update_assets(&mut self, asset_ids: Vec<String>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError> {
        Ok(AssetsStore::update_assets(self, asset_ids, updates)?)
    }

    fn update_asset(&mut self, asset_id: String, update: AssetUpdate) -> Result<usize, DatabaseError> {
        Ok(AssetsStore::update_asset(self, asset_id, update)?)
    }

    fn upsert_assets(&mut self, values: Vec<PrimitiveAsset>) -> Result<usize, DatabaseError> {
        Ok(AssetsStore::upsert_assets(
            self,
            values.into_iter().map(Asset::from_primitive_default).collect(),
        )?)
    }

    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetBasic>, DatabaseError> {
        Ok(AssetsStore::get_assets_by_filter(self, filters)?
            .into_iter()
            .map(|x| x.as_basic_primitive())
            .collect())
    }

    fn get_asset(&mut self, asset_id: &str) -> Result<PrimitiveAsset, DatabaseError> {
        Ok(AssetsStore::get_asset(self, asset_id)?.as_primitive())
    }

    fn get_asset_full(&mut self, asset_id: &str) -> Result<primitives::AssetFull, DatabaseError> {
        use crate::database::assets_links::AssetsLinksStore;
        use crate::database::prices::PricesStore;
        use crate::database::tag::TagStore;

        let asset = AssetsStore::get_asset(self, asset_id)?;
        let price = PricesStore::get_price(self, asset_id)?;
        let market = price.as_ref().map(|x| x.as_market_primitive());
        let links = AssetsLinksStore::get_asset_links(self, asset_id)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect();
        let tags = TagStore::get_assets_tags_for_asset(self, asset_id)?.into_iter().map(|x| x.tag_id).collect();
        let perpetuals = self.perpetuals().get_perpetuals_for_asset(asset.id.as_str())?;
        let perpetuals = perpetuals.into_iter().map(|x| x.as_basic()).collect();

        Ok(primitives::AssetFull {
            price: price.map(|x| x.as_primitive()),
            market,
            asset: asset.as_primitive(),
            properties: asset.as_property_primitive(),
            score: asset.as_score_primitive(),
            links,
            tags,
            perpetuals,
        })
    }

    fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<PrimitiveAsset>, DatabaseError> {
        Ok(AssetsStore::get_assets(self, asset_ids)?.into_iter().map(|x| x.as_primitive()).collect())
    }

    fn get_assets_basic(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetBasic>, DatabaseError> {
        Ok(AssetsStore::get_assets(self, asset_ids)?.into_iter().map(|x| x.as_basic_primitive()).collect())
    }

    fn get_swap_assets(&mut self) -> Result<Vec<String>, DatabaseError> {
        Ok(AssetsStore::get_swap_assets(self)?)
    }

    fn get_swap_assets_version(&mut self) -> Result<i32, DatabaseError> {
        Ok(AssetsStore::get_swap_assets_version(self)?)
    }

    fn add_chains(&mut self, values: Vec<String>) -> Result<usize, DatabaseError> {
        Ok(AssetsStore::add_chains(self, values)?)
    }
}

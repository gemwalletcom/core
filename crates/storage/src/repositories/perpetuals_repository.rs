use crate::database::perpetuals::{PerpetualFilter, PerpetualsStore};
use crate::models::{NewPerpetualRow, PerpetualRow};
use crate::{DatabaseClient, DatabaseError};
use primitives::{AssetId, perpetual::Perpetual};

pub trait PerpetualsRepository {
    fn get_perpetuals_for_asset(&mut self, asset_id: &AssetId) -> Result<Vec<Perpetual>, DatabaseError>;

    fn perpetuals_update(&mut self, values: Vec<NewPerpetualRow>) -> Result<usize, DatabaseError>;

    fn perpetuals_all(&mut self) -> Result<Vec<Perpetual>, DatabaseError>;

    fn get_perpetuals_by_filter(&mut self, filters: Vec<PerpetualFilter>) -> Result<Vec<PerpetualRow>, DatabaseError>;
}

impl PerpetualsRepository for DatabaseClient {
    fn get_perpetuals_for_asset(&mut self, asset_id: &AssetId) -> Result<Vec<Perpetual>, DatabaseError> {
        Ok(PerpetualsStore::get_perpetuals_for_asset(self, &asset_id.to_string())?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect())
    }

    fn perpetuals_update(&mut self, values: Vec<NewPerpetualRow>) -> Result<usize, DatabaseError> {
        Ok(PerpetualsStore::perpetuals_update(self, values)?)
    }

    fn perpetuals_all(&mut self) -> Result<Vec<Perpetual>, DatabaseError> {
        Ok(PerpetualsStore::get_perpetuals_by_filter(self, vec![])?.into_iter().map(|x| x.as_primitive()).collect())
    }

    fn get_perpetuals_by_filter(&mut self, filters: Vec<PerpetualFilter>) -> Result<Vec<PerpetualRow>, DatabaseError> {
        Ok(PerpetualsStore::get_perpetuals_by_filter(self, filters)?)
    }
}

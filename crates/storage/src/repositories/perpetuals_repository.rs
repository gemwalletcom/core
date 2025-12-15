use crate::database::perpetuals::PerpetualsStore;
use crate::models::PerpetualRow;
use crate::{DatabaseClient, DatabaseError};
use primitives::perpetual::Perpetual;

pub trait PerpetualsRepository {
    fn get_perpetuals_for_asset(&mut self, asset_id: &str) -> Result<Vec<Perpetual>, DatabaseError>;

    fn perpetuals_update(&mut self, values: Vec<PerpetualRow>) -> Result<usize, DatabaseError>;

    fn perpetuals_all(&mut self) -> Result<Vec<Perpetual>, DatabaseError>;
}

impl PerpetualsRepository for DatabaseClient {
    fn get_perpetuals_for_asset(&mut self, asset_id: &str) -> Result<Vec<Perpetual>, DatabaseError> {
        Ok(PerpetualsStore::get_perpetuals_for_asset(self, asset_id)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect())
    }

    fn perpetuals_update(&mut self, values: Vec<PerpetualRow>) -> Result<usize, DatabaseError> {
        Ok(PerpetualsStore::perpetuals_update(self, values)?)
    }

    fn perpetuals_all(&mut self) -> Result<Vec<Perpetual>, DatabaseError> {
        Ok(PerpetualsStore::get_all_perpetuals(self)?.into_iter().map(|x| x.as_primitive()).collect())
    }
}

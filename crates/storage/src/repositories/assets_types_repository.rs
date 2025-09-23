use crate::DatabaseError;

use crate::DatabaseClient;
use crate::database::assets_types::AssetsTypesStore;

pub trait AssetsTypesRepository {
    fn add_assets_types(&mut self, values: Vec<primitives::AssetType>) -> Result<usize, DatabaseError>;
}

impl AssetsTypesRepository for DatabaseClient {
    fn add_assets_types(&mut self, values: Vec<primitives::AssetType>) -> Result<usize, DatabaseError> {
        let storage_values = values
            .iter()
            .map(|x| crate::models::AssetType { id: x.as_ref().to_owned() })
            .collect::<Vec<_>>();
        Ok(AssetsTypesStore::add_assets_types(self, storage_values)?)
    }
}

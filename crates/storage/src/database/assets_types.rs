use crate::schema::assets_types::dsl::*;

use crate::{models::AssetType, DatabaseClient};
use diesel::prelude::*;

pub(crate) trait AssetsTypesStore {
    fn add_assets_types(&mut self, values: Vec<AssetType>) -> Result<usize, diesel::result::Error>;
}

impl AssetsTypesStore for DatabaseClient {
    fn add_assets_types(&mut self, values: Vec<AssetType>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(assets_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

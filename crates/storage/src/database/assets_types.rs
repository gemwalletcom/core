use crate::schema::assets_types::dsl::*;

use crate::{models::AssetType, DatabaseClient};
use diesel::prelude::*;

impl DatabaseClient {
    pub fn add_assets_types(&mut self, values: Vec<primitives::AssetType>) -> Result<usize, diesel::result::Error> {
        let values = values.iter().map(|x| AssetType { id: x.as_ref().to_owned() }).collect::<Vec<_>>();

        diesel::insert_into(assets_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

use crate::models::asset::AssetLink;
use crate::schema::assets_links::dsl::*;

use crate::DatabaseClient;
use diesel::{prelude::*, upsert::excluded};

impl DatabaseClient {
    pub fn add_assets_links(&mut self, values: Vec<AssetLink>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(assets_links)
            .values(values)
            .on_conflict((asset_id, link_type))
            .do_update()
            .set((url.eq(excluded(url)),))
            .execute(&mut self.connection)
    }
}

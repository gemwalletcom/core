use crate::models::asset::AssetLink;
use crate::schema::assets_links::dsl::*;

use crate::DatabaseClient;
use diesel::{prelude::*, upsert::excluded};

pub(crate) trait AssetsLinksStore {
    fn add_assets_links(&mut self, values: Vec<AssetLink>) -> Result<usize, diesel::result::Error>;
    fn get_asset_links(&mut self, asset_id: &str) -> Result<Vec<AssetLink>, diesel::result::Error>;
}

impl AssetsLinksStore for DatabaseClient {
    fn add_assets_links(&mut self, values: Vec<AssetLink>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(assets_links)
            .values(values)
            .on_conflict((asset_id, link_type))
            .do_update()
            .set((url.eq(excluded(url)),))
            .execute(&mut self.connection)
    }

    fn get_asset_links(&mut self, _asset_id: &str) -> Result<Vec<AssetLink>, diesel::result::Error> {
        use crate::schema::assets_links::dsl::*;
        assets_links
            .filter(asset_id.eq(_asset_id))
            .select(AssetLink::as_select())
            .load(&mut self.connection)
    }
}

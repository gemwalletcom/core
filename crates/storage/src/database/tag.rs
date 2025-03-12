use crate::{models::*, DatabaseClient};
use diesel::prelude::*;

impl DatabaseClient {
    pub fn add_tags(&mut self, values: Vec<Tag>) -> Result<usize, diesel::result::Error> {
        use crate::schema::tags::dsl::*;
        diesel::insert_into(tags).values(values).on_conflict_do_nothing().execute(&mut self.connection)
    }

    pub fn add_assets_tags(&mut self, values: Vec<AssetTag>) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets_tags::dsl::*;
        diesel::insert_into(assets_tags)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn get_assets_tags(&mut self) -> Result<Vec<AssetTag>, diesel::result::Error> {
        use crate::schema::assets_tags::dsl::*;
        assets_tags.select(AssetTag::as_select()).load(&mut self.connection)
    }

    pub fn get_assets_tags_for_asset(&mut self, _asset_id: &str) -> Result<Vec<AssetTag>, diesel::result::Error> {
        use crate::schema::assets_tags::dsl::*;
        assets_tags
            .filter(asset_id.eq(_asset_id))
            .select(AssetTag::as_select())
            .load(&mut self.connection)
    }
}

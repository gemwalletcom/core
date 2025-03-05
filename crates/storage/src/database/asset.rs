use crate::{
    models::{asset::AssetLink, Asset, AssetType},
    DatabaseClient,
};
use diesel::{prelude::*, upsert::excluded};

impl DatabaseClient {
    pub fn add_assets(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        diesel::insert_into(assets)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn update_assets_rank(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        diesel::insert_into(assets)
            .values(values)
            .on_conflict(id)
            .do_update()
            .set((rank.eq(excluded(rank)),))
            .execute(&mut self.connection)
    }

    pub fn add_assets_types(&mut self, values: Vec<primitives::AssetType>) -> Result<usize, diesel::result::Error> {
        let values = values.iter().map(|x| AssetType { id: x.as_ref().to_owned() }).collect::<Vec<_>>();

        use crate::schema::assets_types::dsl::*;
        diesel::insert_into(assets_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_assets_links(&mut self, values: Vec<AssetLink>) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets_links::dsl::*;
        diesel::insert_into(assets_links)
            .values(values)
            .on_conflict((asset_id, link_type))
            .do_update()
            .set((url.eq(excluded(url)),))
            .execute(&mut self.connection)
    }

    pub fn get_assets_list(&mut self) -> Result<Vec<Asset>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(is_enabled.eq(true)).select(Asset::as_select()).load(&mut self.connection)
    }
}

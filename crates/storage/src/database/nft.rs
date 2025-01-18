use crate::{models::*, DatabaseClient};

use diesel::{prelude::*, upsert::excluded};
use nft_asset::NftLink;

impl DatabaseClient {
    pub fn get_nft_collections(&mut self) -> Result<Vec<NftCollection>, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        nft_collections.select(NftCollection::as_select()).load(&mut self.connection)
    }

    pub fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollection, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        nft_collections
            .filter(id.eq(collection_id))
            .select(NftCollection::as_select())
            .first(&mut self.connection)
    }

    pub fn add_nft_collections(&mut self, values: Vec<NftCollection>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        diesel::insert_into(nft_collections)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_nft_assets(&mut self, values: Vec<NftAsset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        diesel::insert_into(nft_assets)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_nft_types(&mut self, values: Vec<NftType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_types::dsl::*;
        diesel::insert_into(nft_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_nft_links(&mut self, values: Vec<NftLink>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_links::dsl::*;
        diesel::insert_into(nft_links)
            .values(values)
            .on_conflict((collection_id, link_type))
            .do_update()
            .set((url.eq(excluded(url)),))
            .execute(&mut self.connection)
    }
}

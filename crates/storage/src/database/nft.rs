use crate::{models::*, DatabaseClient};

use diesel::prelude::*;
use nft_asset::UpdateNftAssetImageUrl;
use nft_collection::UpdateNftCollectionImageUrl;
use nft_link::NftLink;

impl DatabaseClient {
    // assets

    pub fn get_nft_assets_all(&mut self) -> Result<Vec<NftAsset>, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        nft_assets.select(NftAsset::as_select()).load(&mut self.connection)
    }

    pub fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAsset>, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        nft_assets.filter(id.eq_any(asset_ids)).select(NftAsset::as_select()).load(&mut self.connection)
    }

    pub fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAsset, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        nft_assets.filter(id.eq(asset_id)).select(NftAsset::as_select()).first(&mut self.connection)
    }

    pub fn add_nft_assets(&mut self, values: Vec<NftAsset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        diesel::insert_into(nft_assets)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrl) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        diesel::update(nft_assets.filter(id.eq(update.id.clone())))
            .set(update)
            .execute(&mut self.connection)
    }

    // collections
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

    pub fn get_nft_collection_links(&mut self, _collection_id: &str) -> Result<Vec<NftLink>, diesel::result::Error> {
        use crate::schema::nft_links::dsl::*;
        nft_links
            .filter(collection_id.eq(_collection_id))
            .select(NftLink::as_select())
            .load(&mut self.connection)
    }

    pub fn add_nft_collections(&mut self, values: Vec<NftCollection>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        diesel::insert_into(nft_collections)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrl) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        diesel::update(nft_collections.filter(id.eq(update.id.clone())))
            .set(update)
            .execute(&mut self.connection)
    }

    // nft types
    pub fn add_nft_types(&mut self, values: Vec<NftType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_types::dsl::*;
        diesel::insert_into(nft_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    // nft links
    pub fn add_nft_links(&mut self, values: Vec<NftLink>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_links::dsl::*;
        diesel::insert_into(nft_links)
            .values(values)
            .on_conflict((collection_id, link_type))
            .do_nothing()
            .execute(&mut self.connection)
    }
}

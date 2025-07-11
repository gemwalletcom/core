use crate::{models::*, DatabaseClient};

use diesel::prelude::*;
use nft_asset::UpdateNftAssetImageUrl;
use nft_collection::UpdateNftCollectionImageUrl;
use nft_link::NftLink;

pub(crate) trait NftStore {
    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAsset>, diesel::result::Error>;
    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAsset, diesel::result::Error>;
    fn add_nft_assets(&mut self, values: Vec<NftAsset>) -> Result<usize, diesel::result::Error>;
    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrl) -> Result<usize, diesel::result::Error>;
    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollection>, diesel::result::Error>;
    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollection, diesel::result::Error>;
    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollection>, diesel::result::Error>;
    fn get_nft_collection_links(&mut self, collection_id: &str) -> Result<Vec<NftLink>, diesel::result::Error>;
    fn add_nft_collections(&mut self, values: Vec<NftCollection>) -> Result<usize, diesel::result::Error>;
    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrl) -> Result<usize, diesel::result::Error>;
    fn add_nft_types(&mut self, values: Vec<NftType>) -> Result<usize, diesel::result::Error>;
    fn add_nft_collections_links(&mut self, values: Vec<NftLink>) -> Result<usize, diesel::result::Error>;
}

impl NftStore for DatabaseClient {
    // assets

    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAsset>, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        nft_assets.filter(id.eq_any(asset_ids)).select(NftAsset::as_select()).load(&mut self.connection)
    }

    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAsset, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        nft_assets.filter(id.eq(asset_id)).select(NftAsset::as_select()).first(&mut self.connection)
    }

    fn add_nft_assets(&mut self, values: Vec<NftAsset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        diesel::insert_into(nft_assets)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrl) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        diesel::update(nft_assets.filter(id.eq(update.id.clone())))
            .set(update)
            .execute(&mut self.connection)
    }

    // collections
    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollection>, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        nft_collections.select(NftCollection::as_select()).load(&mut self.connection)
    }

    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollection, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        nft_collections
            .filter(id.eq(collection_id))
            .select(NftCollection::as_select())
            .first(&mut self.connection)
    }

    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollection>, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        nft_collections
            .filter(id.eq_any(ids))
            .select(NftCollection::as_select())
            .load(&mut self.connection)
    }

    fn get_nft_collection_links(&mut self, _collection_id: &str) -> Result<Vec<NftLink>, diesel::result::Error> {
        use crate::schema::nft_collections_links::dsl::*;
        nft_collections_links
            .filter(collection_id.eq(_collection_id))
            .select(NftLink::as_select())
            .load(&mut self.connection)
    }

    fn add_nft_collections(&mut self, values: Vec<NftCollection>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        diesel::insert_into(nft_collections)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrl) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        diesel::update(nft_collections.filter(id.eq(update.id.clone())))
            .set(update)
            .execute(&mut self.connection)
    }

    // nft types
    fn add_nft_types(&mut self, values: Vec<NftType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_types::dsl::*;
        diesel::insert_into(nft_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    // nft links
    fn add_nft_collections_links(&mut self, values: Vec<NftLink>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections_links::dsl::*;
        diesel::insert_into(nft_collections_links)
            .values(values)
            .on_conflict((collection_id, link_type))
            .do_nothing()
            .execute(&mut self.connection)
    }
}

use crate::{DatabaseClient, models::*};

use diesel::prelude::*;
use nft_asset::UpdateNftAssetImageUrlRow;
use nft_collection::UpdateNftCollectionImageUrlRow;
use nft_link::NftLinkRow;
use nft_report::NewNftReportRow;

pub(crate) trait NftStore {
    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAssetRow>, diesel::result::Error>;
    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAssetRow, diesel::result::Error>;
    fn add_nft_assets(&mut self, values: Vec<NftAssetRow>) -> Result<usize, diesel::result::Error>;
    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrlRow) -> Result<usize, diesel::result::Error>;
    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollectionRow>, diesel::result::Error>;
    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollectionRow, diesel::result::Error>;
    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollectionRow>, diesel::result::Error>;
    fn get_nft_collection_links(&mut self, collection_id: &str) -> Result<Vec<NftLinkRow>, diesel::result::Error>;
    fn add_nft_collections(&mut self, values: Vec<NftCollectionRow>) -> Result<usize, diesel::result::Error>;
    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrlRow) -> Result<usize, diesel::result::Error>;
    fn add_nft_collections_links(&mut self, values: Vec<NftLinkRow>) -> Result<usize, diesel::result::Error>;
    fn add_nft_report(&mut self, report: NewNftReportRow) -> Result<usize, diesel::result::Error>;
}

impl NftStore for DatabaseClient {
    // assets

    fn get_nft_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<NftAssetRow>, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        nft_assets
            .filter(id.eq_any(asset_ids))
            .select(NftAssetRow::as_select())
            .load(&mut self.connection)
    }

    fn get_nft_asset(&mut self, asset_id: &str) -> Result<NftAssetRow, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        nft_assets.find(asset_id).select(NftAssetRow::as_select()).first(&mut self.connection)
    }

    fn add_nft_assets(&mut self, values: Vec<NftAssetRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        diesel::insert_into(nft_assets)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn update_nft_asset_image_url(&mut self, update: UpdateNftAssetImageUrlRow) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        diesel::update(nft_assets.find(update.id.clone())).set(update).execute(&mut self.connection)
    }

    // collections
    fn get_nft_collections_all(&mut self) -> Result<Vec<NftCollectionRow>, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        nft_collections.select(NftCollectionRow::as_select()).load(&mut self.connection)
    }

    fn get_nft_collection(&mut self, collection_id: &str) -> Result<NftCollectionRow, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        nft_collections
            .find(collection_id)
            .select(NftCollectionRow::as_select())
            .first(&mut self.connection)
    }

    fn get_nft_collections(&mut self, ids: Vec<String>) -> Result<Vec<NftCollectionRow>, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        nft_collections
            .filter(id.eq_any(ids))
            .select(NftCollectionRow::as_select())
            .load(&mut self.connection)
    }

    fn get_nft_collection_links(&mut self, _collection_id: &str) -> Result<Vec<NftLinkRow>, diesel::result::Error> {
        use crate::schema::nft_collections_links::dsl::*;
        nft_collections_links
            .filter(collection_id.eq(_collection_id))
            .select(NftLinkRow::as_select())
            .load(&mut self.connection)
    }

    fn add_nft_collections(&mut self, values: Vec<NftCollectionRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        diesel::insert_into(nft_collections)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn update_nft_collection_image_url(&mut self, update: UpdateNftCollectionImageUrlRow) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        diesel::update(nft_collections.find(update.id.clone()))
            .set(update)
            .execute(&mut self.connection)
    }

    // nft links
    fn add_nft_collections_links(&mut self, values: Vec<NftLinkRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections_links::dsl::*;
        diesel::insert_into(nft_collections_links)
            .values(values)
            .on_conflict((collection_id, link_type))
            .do_nothing()
            .execute(&mut self.connection)
    }

    // nft reports
    fn add_nft_report(&mut self, report: NewNftReportRow) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_reports::dsl::*;
        diesel::insert_into(nft_reports)
            .values(report)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}

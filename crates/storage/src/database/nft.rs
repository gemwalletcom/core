use crate::sql_types::ChainRow;
use crate::{DatabaseClient, models::*};

use chrono::NaiveDateTime;
use diesel::prelude::*;
use nft_asset::NewNftAssetRow;
use nft_asset_association::NewNftAssetAssociationRow;
use nft_link::NftLinkRow;
use nft_report::NewNftReportRow;
use primitives::Chain;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NftCollectionFilter {
    UpdatedSince(NaiveDateTime),
    Ids(Vec<i32>),
    Identifiers(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NftAssetFilter {
    Identifiers(Vec<String>),
    AddressId(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NftAssetAssociationFilter {
    AddressId(i32),
    Chains(Vec<Chain>),
}

pub(crate) trait NftStore {
    fn get_nft_assets_by_filter(&mut self, filters: Vec<NftAssetFilter>) -> Result<Vec<NftAssetRow>, diesel::result::Error>;
    fn get_nft_asset(&mut self, identifier: &str) -> Result<NftAssetRow, diesel::result::Error>;
    fn add_nft_assets(&mut self, values: Vec<NewNftAssetRow>) -> Result<usize, diesel::result::Error>;
    fn get_nft_collections_by_filter(&mut self, filters: Vec<NftCollectionFilter>) -> Result<Vec<NftCollectionRow>, diesel::result::Error>;
    fn get_nft_collection(&mut self, identifier: &str) -> Result<NftCollectionRow, diesel::result::Error>;
    fn get_nft_collection_links(&mut self, collection_id: i32) -> Result<Vec<NftLinkRow>, diesel::result::Error>;
    fn add_nft_collections(&mut self, values: Vec<NewNftCollectionRow>) -> Result<usize, diesel::result::Error>;
    fn add_nft_collections_links(&mut self, values: Vec<NftLinkRow>) -> Result<usize, diesel::result::Error>;
    fn add_nft_report(&mut self, report: NewNftReportRow) -> Result<usize, diesel::result::Error>;
    fn get_nft_asset_association_ids_by_filter(&mut self, filters: Vec<NftAssetAssociationFilter>) -> Result<Vec<i32>, diesel::result::Error>;
    fn add_nft_asset_associations(&mut self, values: Vec<NewNftAssetAssociationRow>) -> Result<usize, diesel::result::Error>;
    fn delete_nft_asset_associations(&mut self, address_id: i32, asset_ids: Vec<i32>) -> Result<usize, diesel::result::Error>;
}

impl NftStore for DatabaseClient {
    fn get_nft_assets_by_filter(&mut self, filters: Vec<NftAssetFilter>) -> Result<Vec<NftAssetRow>, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        use crate::schema::nft_assets_associations;

        let mut query = nft_assets.into_boxed();
        for filter in filters {
            match filter {
                NftAssetFilter::Identifiers(values) => query = query.filter(identifier.eq_any(values)),
                NftAssetFilter::AddressId(value) => {
                    query = query.filter(
                        id.eq_any(
                            nft_assets_associations::table
                                .filter(nft_assets_associations::address_id.eq(value))
                                .select(nft_assets_associations::asset_id),
                        ),
                    );
                }
            }
        }
        query.select(NftAssetRow::as_select()).load(&mut self.connection)
    }

    fn get_nft_asset(&mut self, _identifier: &str) -> Result<NftAssetRow, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        nft_assets.filter(identifier.eq(_identifier)).select(NftAssetRow::as_select()).first(&mut self.connection)
    }

    fn add_nft_assets(&mut self, values: Vec<NewNftAssetRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::*;
        diesel::insert_into(nft_assets).values(values).on_conflict_do_nothing().execute(&mut self.connection)
    }

    fn get_nft_collections_by_filter(&mut self, filters: Vec<NftCollectionFilter>) -> Result<Vec<NftCollectionRow>, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        let mut query = nft_collections.into_boxed();
        for filter in filters {
            match filter {
                NftCollectionFilter::UpdatedSince(value) => query = query.filter(updated_at.gt(value)),
                NftCollectionFilter::Ids(values) => query = query.filter(id.eq_any(values)),
                NftCollectionFilter::Identifiers(values) => query = query.filter(identifier.eq_any(values)),
            }
        }
        query.select(NftCollectionRow::as_select()).load(&mut self.connection)
    }

    fn get_nft_collection(&mut self, _identifier: &str) -> Result<NftCollectionRow, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        nft_collections
            .filter(identifier.eq(_identifier))
            .select(NftCollectionRow::as_select())
            .first(&mut self.connection)
    }

    fn get_nft_collection_links(&mut self, _collection_id: i32) -> Result<Vec<NftLinkRow>, diesel::result::Error> {
        use crate::schema::nft_collections_links::dsl::*;
        nft_collections_links
            .filter(collection_id.eq(_collection_id))
            .select(NftLinkRow::as_select())
            .load(&mut self.connection)
    }

    fn add_nft_collections(&mut self, values: Vec<NewNftCollectionRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections::dsl::*;
        diesel::insert_into(nft_collections).values(values).on_conflict_do_nothing().execute(&mut self.connection)
    }

    fn add_nft_collections_links(&mut self, values: Vec<NftLinkRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_collections_links::dsl::*;
        diesel::insert_into(nft_collections_links)
            .values(values)
            .on_conflict((collection_id, link_type))
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn add_nft_report(&mut self, report: NewNftReportRow) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_reports::dsl::*;
        diesel::insert_into(nft_reports).values(report).on_conflict_do_nothing().execute(&mut self.connection)
    }

    fn get_nft_asset_association_ids_by_filter(&mut self, filters: Vec<NftAssetAssociationFilter>) -> Result<Vec<i32>, diesel::result::Error> {
        use crate::schema::nft_assets::dsl::{chain as asset_chain, id as asset_pk, nft_assets};
        use crate::schema::nft_assets_associations::dsl::*;
        let mut query = nft_assets_associations.inner_join(nft_assets.on(asset_pk.eq(asset_id))).into_boxed();
        for filter in filters {
            match filter {
                NftAssetAssociationFilter::AddressId(value) => query = query.filter(address_id.eq(value)),
                NftAssetAssociationFilter::Chains(values) => {
                    query = query.filter(asset_chain.eq_any(values.into_iter().map(ChainRow::from).collect::<Vec<_>>()))
                }
            }
        }
        query.select(asset_id).load(&mut self.connection)
    }

    fn add_nft_asset_associations(&mut self, values: Vec<NewNftAssetAssociationRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets_associations::dsl::*;
        diesel::insert_into(nft_assets_associations)
            .values(values)
            .on_conflict((address_id, asset_id))
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn delete_nft_asset_associations(&mut self, _address_id: i32, asset_ids: Vec<i32>) -> Result<usize, diesel::result::Error> {
        use crate::schema::nft_assets_associations::dsl::*;
        diesel::delete(nft_assets_associations.filter(address_id.eq(_address_id)).filter(asset_id.eq_any(asset_ids))).execute(&mut self.connection)
    }
}

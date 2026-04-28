use crate::database::nft::{NftAssetAssociationFilter, NftAssetFilter, NftCollectionFilter, NftStore};
use crate::models::{NewNftAssetAssociationRow, NewNftAssetRow, NewNftCollectionRow, NftAssetRow, NftCollectionRow, nft_link::NftLinkRow, nft_report::NewNftReportRow};
use crate::{DatabaseClient, DatabaseError, DieselResultExt};
use primitives::{Chain, Diff};

pub trait NftRepository {
    fn get_nft_assets_by_filter(&mut self, filters: Vec<NftAssetFilter>) -> Result<Vec<NftAssetRow>, DatabaseError>;
    fn get_nft_asset(&mut self, identifier: &str) -> Result<NftAssetRow, DatabaseError>;
    fn add_nft_assets(&mut self, values: Vec<NewNftAssetRow>) -> Result<usize, DatabaseError>;
    fn get_nft_collections_by_filter(&mut self, filters: Vec<NftCollectionFilter>) -> Result<Vec<NftCollectionRow>, DatabaseError>;
    fn get_nft_collection(&mut self, identifier: &str) -> Result<NftCollectionRow, DatabaseError>;
    fn get_nft_collection_links(&mut self, collection_id: i32) -> Result<Vec<NftLinkRow>, DatabaseError>;
    fn add_nft_collections(&mut self, values: Vec<NewNftCollectionRow>) -> Result<usize, DatabaseError>;
    fn add_nft_collections_links(&mut self, values: Vec<NftLinkRow>) -> Result<usize, DatabaseError>;
    fn add_nft_report(&mut self, report: NewNftReportRow) -> Result<usize, DatabaseError>;
    fn set_nft_asset_associations(&mut self, address_id: i32, chains: Vec<Chain>, asset_ids: Vec<i32>) -> Result<(), DatabaseError>;
}

impl NftRepository for DatabaseClient {
    fn get_nft_assets_by_filter(&mut self, filters: Vec<NftAssetFilter>) -> Result<Vec<NftAssetRow>, DatabaseError> {
        Ok(NftStore::get_nft_assets_by_filter(self, filters)?)
    }

    fn get_nft_asset(&mut self, identifier: &str) -> Result<NftAssetRow, DatabaseError> {
        NftStore::get_nft_asset(self, identifier).or_not_found(identifier.to_string())
    }

    fn add_nft_assets(&mut self, values: Vec<NewNftAssetRow>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_assets(self, values)?)
    }

    fn get_nft_collections_by_filter(&mut self, filters: Vec<NftCollectionFilter>) -> Result<Vec<NftCollectionRow>, DatabaseError> {
        Ok(NftStore::get_nft_collections_by_filter(self, filters)?)
    }

    fn get_nft_collection(&mut self, identifier: &str) -> Result<NftCollectionRow, DatabaseError> {
        NftStore::get_nft_collection(self, identifier).or_not_found(identifier.to_string())
    }

    fn get_nft_collection_links(&mut self, collection_id: i32) -> Result<Vec<NftLinkRow>, DatabaseError> {
        Ok(NftStore::get_nft_collection_links(self, collection_id)?)
    }

    fn add_nft_collections(&mut self, values: Vec<NewNftCollectionRow>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_collections(self, values)?)
    }

    fn add_nft_collections_links(&mut self, values: Vec<NftLinkRow>) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_collections_links(self, values)?)
    }

    fn add_nft_report(&mut self, report: NewNftReportRow) -> Result<usize, DatabaseError> {
        Ok(NftStore::add_nft_report(self, report)?)
    }

    fn set_nft_asset_associations(&mut self, address_id: i32, chains: Vec<Chain>, asset_ids: Vec<i32>) -> Result<(), DatabaseError> {
        let existing = NftStore::get_nft_asset_association_ids_by_filter(self, vec![NftAssetAssociationFilter::AddressId(address_id), NftAssetAssociationFilter::Chains(chains)])?;
        let diff = Diff::compare(asset_ids, existing);

        let to_insert: Vec<NewNftAssetAssociationRow> = diff.different.into_iter().map(|asset_id| NewNftAssetAssociationRow { address_id, asset_id }).collect();

        if !to_insert.is_empty() {
            NftStore::add_nft_asset_associations(self, to_insert)?;
        }
        if !diff.missing.is_empty() {
            NftStore::delete_nft_asset_associations(self, address_id, diff.missing)?;
        }
        Ok(())
    }
}

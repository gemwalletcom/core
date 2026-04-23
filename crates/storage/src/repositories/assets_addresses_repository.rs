use crate::database::assets_addresses::AssetsAddressesStore;
use crate::models::{AssetAddressRow, AssetAddressRowsExt};
use crate::{DatabaseClient, DatabaseError};
use chrono::NaiveDateTime;
use primitives::{AssetAddress as PrimitiveAssetAddress, AssetId, ChainAddress};

pub trait AssetsAddressesRepository {
    fn add_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError>;
    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_datetime: Option<NaiveDateTime>) -> Result<Vec<AssetId>, DatabaseError>;
    fn get_asset_addresses(&mut self, value: ChainAddress) -> Result<Vec<PrimitiveAssetAddress>, DatabaseError>;
    fn get_asset_address(&mut self, value: ChainAddress, asset_id: AssetId) -> Result<Option<PrimitiveAssetAddress>, DatabaseError>;
    fn delete_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError>;
}

impl AssetsAddressesRepository for DatabaseClient {
    fn add_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError> {
        Ok(AssetsAddressesStore::add_assets_addresses(
            self,
            values.into_iter().map(AssetAddressRow::from_primitive).collect(),
        )?)
    }

    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_datetime: Option<NaiveDateTime>) -> Result<Vec<AssetId>, DatabaseError> {
        Ok(AssetsAddressesStore::get_assets_by_addresses(self, values, from_datetime)?.asset_ids())
    }

    fn get_asset_addresses(&mut self, value: ChainAddress) -> Result<Vec<PrimitiveAssetAddress>, DatabaseError> {
        Ok(AssetsAddressesStore::get_asset_addresses(self, value)?.into_iter().map(|row| row.as_primitive()).collect())
    }

    fn get_asset_address(&mut self, value: ChainAddress, asset_id: AssetId) -> Result<Option<PrimitiveAssetAddress>, DatabaseError> {
        Ok(AssetsAddressesStore::get_asset_address(self, value, asset_id)?.map(|row| row.as_primitive()))
    }

    fn delete_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError> {
        Ok(AssetsAddressesStore::delete_assets_addresses(
            self,
            values.into_iter().map(AssetAddressRow::from_primitive).collect(),
        )?)
    }
}

use crate::database::assets_addresses::AssetsAddressesStore;
use crate::models::asset_address::AssetAddressRow;
use crate::{DatabaseClient, DatabaseError};
use chrono::NaiveDateTime;
use primitives::{AssetAddress as PrimitiveAssetAddress, AssetId, ChainAddress};

pub trait AssetsAddressesRepository {
    fn add_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError>;
    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_datetime: Option<NaiveDateTime>, include_with_prices: bool) -> Result<Vec<AssetId>, DatabaseError>;
    fn delete_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError>;
}

impl AssetsAddressesRepository for DatabaseClient {
    fn add_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError> {
        Ok(AssetsAddressesStore::add_assets_addresses(
            self,
            values.into_iter().map(AssetAddressRow::from_primitive).collect(),
        )?)
    }

    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_datetime: Option<NaiveDateTime>, include_with_prices: bool) -> Result<Vec<AssetId>, DatabaseError> {
        Ok(AssetsAddressesStore::get_assets_by_addresses(self, values, from_datetime, include_with_prices)?
            .into_iter()
            .flat_map(|x| AssetId::new(x.asset_id.as_str()))
            .collect())
    }

    fn delete_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError> {
        Ok(AssetsAddressesStore::delete_assets_addresses(
            self,
            values.into_iter().map(AssetAddressRow::from_primitive).collect(),
        )?)
    }
}

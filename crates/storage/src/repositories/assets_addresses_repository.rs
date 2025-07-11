use std::error::Error;

use crate::{models::asset_address::AssetAddress, DatabaseClient};
use crate::database::assets_addresses::AssetsAddressesStore;
use primitives::{AssetAddress as PrimitiveAssetAddress, AssetId, ChainAddress};

pub trait AssetsAddressesRepository {
    fn add_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_assets_by_addresses(
        &mut self,
        values: Vec<ChainAddress>,
        from_timestamp: Option<u32>,
        include_with_prices: bool,
    ) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>>;
    fn delete_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl AssetsAddressesRepository for DatabaseClient {
    fn add_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsAddressesStore::add_assets_addresses(
            self,
            values.into_iter().map(AssetAddress::from_primitive).collect(),
        )?)
    }

    fn get_assets_by_addresses(
        &mut self,
        values: Vec<ChainAddress>,
        from_timestamp: Option<u32>,
        include_with_prices: bool,
    ) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        Ok(
            AssetsAddressesStore::get_assets_by_addresses(self, values, from_timestamp, include_with_prices)?
                .into_iter()
                .flat_map(|x| AssetId::new(x.asset_id.as_str()))
                .collect(),
        )
    }

    fn delete_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(AssetsAddressesStore::delete_assets_addresses(
            self,
            values.into_iter().map(AssetAddress::from_primitive).collect(),
        )?)
    }
}
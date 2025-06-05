use std::error::Error;

use crate::schema::assets_addresses::dsl::*;

use crate::{models::asset_address::AssetAddress, DatabaseClient};
use chrono::DateTime;
use diesel::prelude::*;
use primitives::{AssetId, ChainAddress};

pub trait AssetsAddressesStore {
    fn add_assets_addresses(&mut self, values: Vec<AssetAddress>) -> Result<usize, diesel::result::Error>;
    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_timestamp: Option<u32>) -> Result<Vec<AssetAddress>, diesel::result::Error>;
}

pub trait AssetsAddressesRepository {
    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_timestamp: Option<u32>) -> Result<Vec<AssetId>, Box<dyn Error>>;
}

impl AssetsAddressesStore for DatabaseClient {
    fn add_assets_addresses(&mut self, values: Vec<AssetAddress>) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(assets_addresses)
            .values(&values)
            .on_conflict((asset_id, address))
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_timestamp: Option<u32>) -> Result<Vec<AssetAddress>, diesel::result::Error> {
        let datetime = if let Some(from_timestamp) = from_timestamp {
            DateTime::from_timestamp(from_timestamp.into(), 0).unwrap().naive_utc()
        } else {
            DateTime::from_timestamp(0, 0).unwrap().naive_utc()
        };
        let chains = values.iter().map(|x| x.chain.as_ref()).collect::<Vec<&str>>();
        let addresses = values.iter().map(|x| x.address.clone()).collect::<Vec<String>>();
        assets_addresses
            .filter(chain.eq_any(chains))
            .filter(address.eq_any(addresses))
            .filter(created_at.gt(datetime))
            .select(AssetAddress::as_select())
            .load(&mut self.connection)
    }
}

impl AssetsAddressesRepository for DatabaseClient {
    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_timestamp: Option<u32>) -> Result<Vec<AssetId>, Box<dyn Error>> {
        Ok(AssetsAddressesStore::get_assets_by_addresses(self, values, from_timestamp)?
            .into_iter()
            .flat_map(|x| AssetId::new(&x.asset_id))
            .collect())
    }
}

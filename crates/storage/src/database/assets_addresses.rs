use crate::schema::assets_addresses::dsl::*;

use crate::{DatabaseClient, models::asset_address::AssetAddress};
use chrono::DateTime;
use diesel::dsl::coalesce;
use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::ChainAddress;

pub(crate) trait AssetsAddressesStore {
    fn add_assets_addresses(&mut self, values: Vec<AssetAddress>) -> Result<usize, diesel::result::Error>;
    fn get_assets_by_addresses(
        &mut self,
        values: Vec<ChainAddress>,
        from_timestamp: Option<u32>,
        include_with_prices: bool,
    ) -> Result<Vec<AssetAddress>, diesel::result::Error>;
    fn delete_assets_addresses(&mut self, values: Vec<AssetAddress>) -> Result<usize, diesel::result::Error>;
}

impl AssetsAddressesStore for DatabaseClient {
    fn add_assets_addresses(&mut self, values: Vec<AssetAddress>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }
        diesel::insert_into(assets_addresses)
            .values(&values)
            .on_conflict((asset_id, address))
            .do_update()
            .set(value.eq(coalesce(excluded(value), value)))
            .execute(&mut self.connection)
    }

    fn get_assets_by_addresses(
        &mut self,
        values: Vec<ChainAddress>,
        from_timestamp: Option<u32>,
        include_with_prices: bool,
    ) -> Result<Vec<AssetAddress>, diesel::result::Error> {
        let datetime = if let Some(from_timestamp) = from_timestamp {
            DateTime::from_timestamp(from_timestamp.into(), 0).unwrap().naive_utc()
        } else {
            DateTime::from_timestamp(0, 0).unwrap().naive_utc()
        };
        let chains = values.iter().map(|x| x.chain.as_ref()).collect::<Vec<&str>>();
        let addresses = values.iter().map(|x| x.address.clone()).collect::<Vec<String>>();
        use crate::schema::assets_addresses::dsl as assets_addresses_dsl;
        use crate::schema::prices_assets::dsl as prices_assets_dsl;

        let mut query = assets_addresses_dsl::assets_addresses
            .filter(assets_addresses_dsl::chain.eq_any(chains))
            .filter(assets_addresses_dsl::address.eq_any(addresses))
            .filter(assets_addresses_dsl::created_at.gt(datetime))
            .select(AssetAddress::as_select())
            .into_boxed();

        if include_with_prices {
            query = query.filter(diesel::dsl::exists(
                prices_assets_dsl::prices_assets.filter(prices_assets_dsl::asset_id.eq(assets_addresses_dsl::asset_id)),
            ));
        }

        query.load(&mut self.connection)
    }

    fn delete_assets_addresses(&mut self, values: Vec<AssetAddress>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }
        let chains = values.iter().map(|x| x.chain.as_ref()).collect::<Vec<&str>>();
        let addresses = values.iter().map(|x| x.address.clone()).collect::<Vec<String>>();
        diesel::delete(assets_addresses.filter(chain.eq_any(chains)).filter(address.eq_any(addresses))).execute(&mut self.connection)
    }
}

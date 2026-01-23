use crate::schema::assets_addresses::dsl::*;

use crate::{DatabaseClient, models::asset_address::AssetAddressRow};
use chrono::NaiveDateTime;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text};
use primitives::ChainAddress;

pub(crate) trait AssetsAddressesStore {
    fn add_assets_addresses(&mut self, values: Vec<AssetAddressRow>) -> Result<usize, diesel::result::Error>;
    fn get_assets_by_addresses(
        &mut self,
        values: Vec<ChainAddress>,
        from_datetime: Option<NaiveDateTime>,
        include_with_prices: bool,
    ) -> Result<Vec<AssetAddressRow>, diesel::result::Error>;
    fn delete_assets_addresses(&mut self, values: Vec<AssetAddressRow>) -> Result<usize, diesel::result::Error>;
}

impl AssetsAddressesStore for DatabaseClient {
    fn add_assets_addresses(&mut self, values: Vec<AssetAddressRow>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }
        diesel::insert_into(assets_addresses)
            .values(&values)
            .on_conflict((asset_id, address))
            .do_update()
            .set(value.eq(sql::<Nullable<Text>>("COALESCE(excluded.value, assets_addresses.value)")))
            .execute(&mut self.connection)
    }

    fn get_assets_by_addresses(
        &mut self,
        values: Vec<ChainAddress>,
        from_datetime: Option<NaiveDateTime>,
        include_with_prices: bool,
    ) -> Result<Vec<AssetAddressRow>, diesel::result::Error> {
        let chains = values.iter().map(|x| x.chain.as_ref()).collect::<Vec<&str>>();
        let addresses = values.iter().map(|x| x.address.clone()).collect::<Vec<String>>();
        use crate::schema::assets_addresses::dsl as assets_addresses_dsl;
        use crate::schema::prices_assets::dsl as prices_assets_dsl;

        let mut query = assets_addresses_dsl::assets_addresses
            .filter(assets_addresses_dsl::chain.eq_any(chains))
            .filter(assets_addresses_dsl::address.eq_any(addresses))
            .select(AssetAddressRow::as_select())
            .into_boxed();

        if let Some(datetime) = from_datetime {
            query = query.filter(assets_addresses_dsl::created_at.gt(datetime));
        }

        if include_with_prices {
            query = query.filter(diesel::dsl::exists(
                prices_assets_dsl::prices_assets.filter(prices_assets_dsl::asset_id.eq(assets_addresses_dsl::asset_id)),
            ));
        }

        query.load(&mut self.connection)
    }

    fn delete_assets_addresses(&mut self, values: Vec<AssetAddressRow>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }
        let chains = values.iter().map(|x| x.chain.as_ref()).collect::<Vec<&str>>();
        let addresses = values.iter().map(|x| x.address.clone()).collect::<Vec<String>>();
        diesel::delete(assets_addresses.filter(chain.eq_any(chains)).filter(address.eq_any(addresses))).execute(&mut self.connection)
    }
}

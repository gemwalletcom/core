use crate::schema::assets_addresses::dsl::*;

use crate::sql_types::AssetId as AssetIdRow;
use crate::{DatabaseClient, models::asset_address::AssetAddressRow};
use chrono::NaiveDateTime;
use diesel::Connection;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text};
use primitives::{AssetId as PrimitiveAssetId, ChainAddress};

pub(crate) trait AssetsAddressesStore {
    fn add_assets_addresses(&mut self, values: Vec<AssetAddressRow>) -> Result<usize, diesel::result::Error>;
    fn get_assets_by_addresses(
        &mut self,
        values: Vec<ChainAddress>,
        from_datetime: Option<NaiveDateTime>,
        prices_min_updated_at: Option<NaiveDateTime>,
    ) -> Result<Vec<AssetAddressRow>, diesel::result::Error>;
    fn get_asset_addresses(&mut self, chain_address: ChainAddress) -> Result<Vec<AssetAddressRow>, diesel::result::Error>;
    fn get_asset_address(&mut self, chain_address: ChainAddress, target_asset_id: PrimitiveAssetId) -> Result<Option<AssetAddressRow>, diesel::result::Error>;
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
        prices_min_updated_at: Option<NaiveDateTime>,
    ) -> Result<Vec<AssetAddressRow>, diesel::result::Error> {
        let chains = values.iter().map(|x| x.chain.as_ref()).collect::<Vec<&str>>();
        let addresses = values.iter().map(|x| x.address.clone()).collect::<Vec<String>>();
        use crate::schema::{assets_addresses::dsl as a, prices, prices_assets, prices_providers};

        let mut query = a::assets_addresses
            .filter(a::chain.eq_any(chains))
            .filter(a::address.eq_any(addresses))
            .filter(a::value.is_null().or(a::value.ne("0")))
            .select(AssetAddressRow::as_select())
            .into_boxed();

        if let Some(datetime) = from_datetime {
            query = query.filter(a::created_at.gt(datetime));
        }

        if let Some(cutoff) = prices_min_updated_at {
            query = query.filter(diesel::dsl::exists(
                prices_assets::table
                    .inner_join(prices::table.on(prices_assets::price_id.eq(prices::id)))
                    .inner_join(prices_providers::table.on(prices_assets::provider.eq(prices_providers::id)))
                    .filter(prices_assets::asset_id.eq(a::asset_id))
                    .filter(prices_providers::enabled.eq(true))
                    .filter(prices::last_updated_at.ge(cutoff)),
            ));
        }

        query.load(&mut self.connection)
    }

    fn get_asset_addresses(&mut self, chain_address: ChainAddress) -> Result<Vec<AssetAddressRow>, diesel::result::Error> {
        assets_addresses
            .filter(chain.eq(chain_address.chain.as_ref()))
            .filter(address.eq(chain_address.address))
            .select(AssetAddressRow::as_select())
            .load(&mut self.connection)
    }

    fn get_asset_address(&mut self, chain_address: ChainAddress, target_asset_id: PrimitiveAssetId) -> Result<Option<AssetAddressRow>, diesel::result::Error> {
        assets_addresses
            .filter(chain.eq(chain_address.chain.as_ref()))
            .filter(address.eq(chain_address.address))
            .filter(asset_id.eq(AssetIdRow::from(target_asset_id)))
            .select(AssetAddressRow::as_select())
            .first(&mut self.connection)
            .optional()
    }

    fn delete_assets_addresses(&mut self, values: Vec<AssetAddressRow>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }

        self.connection.transaction(|connection| {
            let mut deleted = 0;

            for row in values {
                deleted += diesel::delete(
                    assets_addresses
                        .filter(chain.eq(&row.chain))
                        .filter(asset_id.eq(&row.asset_id))
                        .filter(address.eq(&row.address)),
                )
                .execute(connection)?;
            }

            Ok(deleted)
        })
    }
}

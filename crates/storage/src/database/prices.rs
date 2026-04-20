use crate::models::price::NewPriceRow;
use crate::sql_types::PriceProviderRow;
use crate::{DatabaseClient, models::*};
use chrono::NaiveDateTime;
use diesel::upsert::excluded;
use diesel::prelude::*;
use primitives::PriceProvider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetsWithPricesFilter {
    UpdatedSince(NaiveDateTime),
}

#[derive(Debug, Clone)]
pub enum PriceFilter {
    Provider(PriceProvider),
    UpdatedBefore(NaiveDateTime),
    UpdatedAfter(NaiveDateTime),
}

pub(crate) trait PricesStore {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, diesel::result::Error>;
    fn set_prices(&mut self, values: Vec<PriceRow>) -> Result<usize, diesel::result::Error>;
    fn set_prices_assets(&mut self, values: Vec<PriceAssetRow>) -> Result<usize, diesel::result::Error>;
    fn get_prices_by_filter(&mut self, filters: Vec<PriceFilter>) -> Result<Vec<PriceRow>, diesel::result::Error>;
    fn get_prices_assets(&mut self) -> Result<Vec<PriceAssetRow>, diesel::result::Error>;
    fn get_prices_assets_by_provider(&mut self, provider: PriceProvider) -> Result<Vec<PriceAssetRow>, diesel::result::Error>;
    fn get_prices_for_asset_ids(&mut self, asset_ids: &[String]) -> Result<Vec<(String, PriceRow)>, diesel::result::Error>;
    fn get_price_by_id(&mut self, price_id: &str) -> Result<PriceRow, diesel::result::Error>;
    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, diesel::result::Error>;
    fn delete_prices(&mut self, ids: Vec<String>) -> Result<usize, diesel::result::Error>;
    fn get_asset_ids_updated_since(&mut self, since: NaiveDateTime) -> Result<Vec<String>, diesel::result::Error>;
}

impl PricesStore for DatabaseClient {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        diesel::insert_into(prices).values(&values).on_conflict_do_nothing().execute(&mut self.connection)
    }

    fn set_prices(&mut self, values: Vec<PriceRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        diesel::insert_into(prices)
            .values(&values)
            .on_conflict(id)
            .do_update()
            .set((
                price.eq(excluded(price)),
                price_change_percentage_24h.eq(excluded(price_change_percentage_24h)),
                all_time_high.eq(excluded(all_time_high)),
                all_time_high_date.eq(excluded(all_time_high_date)),
                all_time_low.eq(excluded(all_time_low)),
                all_time_low_date.eq(excluded(all_time_low_date)),
                market_cap.eq(excluded(market_cap)),
                market_cap_fdv.eq(excluded(market_cap_fdv)),
                market_cap_rank.eq(excluded(market_cap_rank)),
                total_volume.eq(excluded(total_volume)),
                circulating_supply.eq(excluded(circulating_supply)),
                total_supply.eq(excluded(total_supply)),
                max_supply.eq(excluded(max_supply)),
                last_updated_at.eq(excluded(last_updated_at)),
            ))
            .execute(&mut self.connection)
    }

    fn set_prices_assets(&mut self, values: Vec<PriceAssetRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        diesel::insert_into(prices_assets)
            .values(&values)
            .on_conflict((asset_id, provider))
            .do_update()
            .set(price_id.eq(excluded(price_id)))
            .execute(&mut self.connection)
    }

    fn get_prices_by_filter(&mut self, filters: Vec<PriceFilter>) -> Result<Vec<PriceRow>, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        let query = filters.into_iter().fold(prices.into_boxed(), |q, filter| match filter {
            PriceFilter::Provider(p) => q.filter(provider.eq(PriceProviderRow::from(p))),
            PriceFilter::UpdatedBefore(time) => q.filter(last_updated_at.lt(time).or(last_updated_at.is_null())),
            PriceFilter::UpdatedAfter(time) => q.filter(last_updated_at.ge(time)),
        });
        query.order(market_cap.desc()).select(PriceRow::as_select()).load(&mut self.connection)
    }

    fn get_prices_assets(&mut self) -> Result<Vec<PriceAssetRow>, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        prices_assets.select(PriceAssetRow::as_select()).load(&mut self.connection)
    }

    fn get_prices_assets_by_provider(&mut self, price_provider: PriceProvider) -> Result<Vec<PriceAssetRow>, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        prices_assets
            .filter(provider.eq(PriceProviderRow::from(price_provider)))
            .select(PriceAssetRow::as_select())
            .load(&mut self.connection)
    }

    fn get_prices_for_asset_ids(&mut self, asset_ids: &[String]) -> Result<Vec<(String, PriceRow)>, diesel::result::Error> {
        use crate::schema::{prices, prices_assets};

        prices_assets::table
            .inner_join(prices::table.on(prices_assets::price_id.eq(prices::id)))
            .filter(prices_assets::asset_id.eq_any(asset_ids))
            .select((prices_assets::asset_id, PriceRow::as_select()))
            .load::<(crate::sql_types::AssetId, PriceRow)>(&mut self.connection)
            .map(|rows| rows.into_iter().map(|(id, row)| (id.0.to_string(), row)).collect())
    }

    fn get_price_by_id(&mut self, price_id: &str) -> Result<PriceRow, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        prices.filter(id.eq(price_id)).select(PriceRow::as_select()).first(&mut self.connection)
    }

    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        prices_assets.filter(price_id.eq_any(ids)).select(PriceAssetRow::as_select()).load(&mut self.connection)
    }

    fn delete_prices(&mut self, ids: Vec<String>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        if ids.is_empty() {
            return Ok(0);
        }
        diesel::delete(prices.filter(id.eq_any(ids))).execute(&mut self.connection)
    }

    fn get_asset_ids_updated_since(&mut self, since: NaiveDateTime) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::{prices, prices_assets};

        prices_assets::table
            .inner_join(prices::table.on(prices_assets::price_id.eq(prices::id)))
            .filter(prices::last_updated_at.gt(since))
            .select(prices_assets::asset_id)
            .load::<crate::sql_types::AssetId>(&mut self.connection)
            .map(|ids| ids.into_iter().map(|id| id.0.to_string()).collect())
    }
}

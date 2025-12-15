use crate::models::price::NewPriceRow;
use crate::schema::prices_assets;
use crate::{DatabaseClient, models::*};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::upsert::excluded;
use price::PriceAssetDataRow;

pub(crate) trait PricesStore {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, diesel::result::Error>;
    fn set_prices(&mut self, values: Vec<PriceRow>) -> Result<usize, diesel::result::Error>;
    fn set_prices_assets(&mut self, values: Vec<PriceAssetRow>) -> Result<usize, diesel::result::Error>;
    fn get_prices(&mut self) -> Result<Vec<PriceRow>, diesel::result::Error>;
    fn get_prices_assets(&mut self) -> Result<Vec<PriceAssetRow>, diesel::result::Error>;
    fn get_price(&mut self, asset_id: &str) -> Result<PriceRow, diesel::result::Error>;
    fn get_prices_assets_for_asset_id(&mut self, id: &str) -> Result<Vec<PriceAssetRow>, diesel::result::Error>;
    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, diesel::result::Error>;
    fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, diesel::result::Error>;
    fn get_prices_assets_list(&mut self) -> Result<Vec<PriceAssetDataRow>, diesel::result::Error>;
}

impl PricesStore for DatabaseClient {
    fn add_prices(&mut self, values: Vec<NewPriceRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        diesel::insert_into(prices)
            .values(&values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
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
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn get_prices(&mut self) -> Result<Vec<PriceRow>, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        prices.order(market_cap.desc()).select(PriceRow::as_select()).load(&mut self.connection)
    }

    fn get_prices_assets(&mut self) -> Result<Vec<PriceAssetRow>, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        prices_assets.select(PriceAssetRow::as_select()).load(&mut self.connection)
    }
    fn get_price(&mut self, asset_id: &str) -> Result<PriceRow, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        prices
            .inner_join(prices_assets::table)
            .filter(prices_assets::asset_id.eq(asset_id))
            .select(PriceRow::as_select())
            .first(&mut self.connection)
    }

    fn get_prices_assets_for_asset_id(&mut self, id: &str) -> Result<Vec<PriceAssetRow>, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        prices_assets.filter(asset_id.eq(id)).select(PriceAssetRow::as_select()).load(&mut self.connection)
    }

    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        prices_assets
            .filter(price_id.eq_any(ids))
            .select(PriceAssetRow::as_select())
            .load(&mut self.connection)
    }

    fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        diesel::delete(prices.filter(last_updated_at.lt(time).or(last_updated_at.is_null()))).execute(&mut self.connection)
    }

    fn get_prices_assets_list(&mut self) -> Result<Vec<PriceAssetDataRow>, diesel::result::Error> {
        use crate::schema::{assets, prices, prices_assets};

        let query = assets::table
            .left_join(prices_assets::table.on(prices_assets::asset_id.eq(assets::id)))
            .left_join(prices::table.on(prices_assets::price_id.eq(prices::id)))
            .select((AssetRow::as_select(), Option::<PriceRow>::as_select()));

        query.load::<PriceAssetDataRow>(&mut self.connection)
    }
}

//

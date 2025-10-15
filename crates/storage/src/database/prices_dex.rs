use crate::{DatabaseClient, models::*};
use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::PriceFeedProvider;

pub(crate) trait PricesDexStore {
    fn add_prices_dex_providers(&mut self, values: Vec<PriceDexProvider>) -> Result<usize, diesel::result::Error>;
    fn add_prices_dex(&mut self, values: Vec<PriceDex>) -> Result<usize, diesel::result::Error>;
    fn set_prices_dex(&mut self, values: Vec<PriceDex>) -> Result<usize, diesel::result::Error>;
    fn set_prices_dex_assets(&mut self, values: Vec<PriceDexAsset>) -> Result<usize, diesel::result::Error>;
    fn get_prices_dex(&mut self) -> Result<Vec<PriceDex>, diesel::result::Error>;
    fn get_prices_dex_assets(&mut self) -> Result<Vec<PriceDexAsset>, diesel::result::Error>;
    fn get_prices_dex_by_provider(&mut self, provider: PriceFeedProvider) -> Result<Vec<PriceDex>, diesel::result::Error>;
}

impl PricesDexStore for DatabaseClient {
    fn add_prices_dex_providers(&mut self, values: Vec<PriceDexProvider>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices_dex_providers::dsl::*;
        diesel::insert_into(prices_dex_providers)
            .values(&values)
            .on_conflict(id)
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn add_prices_dex(&mut self, values: Vec<PriceDex>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices_dex::dsl::*;
        diesel::insert_into(prices_dex)
            .values(&values)
            .on_conflict(id)
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn set_prices_dex(&mut self, values: Vec<PriceDex>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices_dex::dsl::*;
        diesel::insert_into(prices_dex)
            .values(&values)
            .on_conflict(id)
            .do_update()
            .set((price.eq(excluded(price)), last_updated_at.eq(excluded(last_updated_at))))
            .execute(&mut self.connection)
    }

    fn set_prices_dex_assets(&mut self, values: Vec<PriceDexAsset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices_dex_assets::dsl::*;
        diesel::insert_into(prices_dex_assets)
            .values(&values)
            .on_conflict((asset_id, price_feed_id))
            .do_nothing()
            .execute(&mut self.connection)
    }

    fn get_prices_dex(&mut self) -> Result<Vec<PriceDex>, diesel::result::Error> {
        use crate::schema::prices_dex::dsl::*;
        prices_dex.select(PriceDex::as_select()).load(&mut self.connection)
    }

    fn get_prices_dex_assets(&mut self) -> Result<Vec<PriceDexAsset>, diesel::result::Error> {
        use crate::schema::prices_dex_assets::dsl::*;
        prices_dex_assets.select(PriceDexAsset::as_select()).load(&mut self.connection)
    }

    fn get_prices_dex_by_provider(&mut self, price_provider: PriceFeedProvider) -> Result<Vec<PriceDex>, diesel::result::Error> {
        use crate::schema::prices_dex::dsl::*;
        prices_dex
            .filter(provider.eq(price_provider.as_ref()))
            .select(PriceDex::as_select())
            .load(&mut self.connection)
    }
}

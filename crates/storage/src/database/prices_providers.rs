use crate::DatabaseClient;
use crate::models::PriceProviderConfigRow;
use diesel::prelude::*;
use diesel::upsert::excluded;

pub(crate) trait PricesProvidersStore {
    fn add_prices_providers(&mut self, values: Vec<PriceProviderConfigRow>) -> Result<usize, diesel::result::Error>;
    fn get_prices_providers(&mut self) -> Result<Vec<PriceProviderConfigRow>, diesel::result::Error>;
}

impl PricesProvidersStore for DatabaseClient {
    fn add_prices_providers(&mut self, values: Vec<PriceProviderConfigRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices_providers::dsl::*;
        diesel::insert_into(prices_providers)
            .values(&values)
            .on_conflict(id)
            .do_update()
            .set((priority.eq(excluded(priority)),))
            .execute(&mut self.connection)
    }

    fn get_prices_providers(&mut self) -> Result<Vec<PriceProviderConfigRow>, diesel::result::Error> {
        use crate::schema::prices_providers::dsl::*;
        prices_providers.order(priority.asc()).select(PriceProviderConfigRow::as_select()).load(&mut self.connection)
    }
}

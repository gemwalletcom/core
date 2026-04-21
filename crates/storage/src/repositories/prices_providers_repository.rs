use crate::DatabaseClient;
use crate::DatabaseError;
use crate::database::prices_providers::PricesProvidersStore;
use crate::models::PriceProviderConfigRow;

pub trait PricesProvidersRepository {
    fn add_prices_providers(&mut self, values: Vec<PriceProviderConfigRow>) -> Result<usize, DatabaseError>;
    fn get_prices_providers(&mut self) -> Result<Vec<PriceProviderConfigRow>, DatabaseError>;
}

impl PricesProvidersRepository for DatabaseClient {
    fn add_prices_providers(&mut self, values: Vec<PriceProviderConfigRow>) -> Result<usize, DatabaseError> {
        Ok(PricesProvidersStore::add_prices_providers(self, values)?)
    }

    fn get_prices_providers(&mut self) -> Result<Vec<PriceProviderConfigRow>, DatabaseError> {
        Ok(PricesProvidersStore::get_prices_providers(self)?)
    }
}

use crate::database::prices_dex::PricesDexStore;
use crate::models::{PriceDex, PriceDexAsset, PriceDexProvider};
use crate::{DatabaseClient, DatabaseError};
use primitives::{PriceFeedId, PriceFeedProvider};

pub trait PricesDexRepository {
    fn add_prices_dex_providers(&mut self, values: Vec<PriceDexProvider>) -> Result<usize, DatabaseError>;
    fn add_prices_dex(&mut self, values: Vec<PriceDex>) -> Result<usize, DatabaseError>;
    fn set_prices_dex(&mut self, values: Vec<PriceDex>) -> Result<usize, DatabaseError>;
    fn set_prices_dex_assets(&mut self, values: Vec<PriceDexAsset>) -> Result<usize, DatabaseError>;
    fn get_prices_dex(&mut self) -> Result<Vec<PriceDex>, DatabaseError>;
    fn get_prices_dex_assets(&mut self) -> Result<Vec<PriceDexAsset>, DatabaseError>;
    fn get_feed_ids_by_provider(&mut self, provider: PriceFeedProvider) -> Result<Vec<PriceFeedId>, DatabaseError>;
}

impl PricesDexRepository for DatabaseClient {
    fn add_prices_dex_providers(&mut self, values: Vec<PriceDexProvider>) -> Result<usize, DatabaseError> {
        Ok(PricesDexStore::add_prices_dex_providers(self, values)?)
    }

    fn add_prices_dex(&mut self, values: Vec<PriceDex>) -> Result<usize, DatabaseError> {
        Ok(PricesDexStore::add_prices_dex(self, values)?)
    }

    fn set_prices_dex(&mut self, values: Vec<PriceDex>) -> Result<usize, DatabaseError> {
        Ok(PricesDexStore::set_prices_dex(self, values)?)
    }

    fn set_prices_dex_assets(&mut self, values: Vec<PriceDexAsset>) -> Result<usize, DatabaseError> {
        Ok(PricesDexStore::set_prices_dex_assets(self, values)?)
    }

    fn get_prices_dex(&mut self) -> Result<Vec<PriceDex>, DatabaseError> {
        Ok(PricesDexStore::get_prices_dex(self)?)
    }

    fn get_prices_dex_assets(&mut self) -> Result<Vec<PriceDexAsset>, DatabaseError> {
        Ok(PricesDexStore::get_prices_dex_assets(self)?)
    }

    fn get_feed_ids_by_provider(&mut self, provider: PriceFeedProvider) -> Result<Vec<PriceFeedId>, DatabaseError> {
        Ok(PricesDexStore::get_prices_dex_by_provider(self, provider)?
            .into_iter()
            .filter_map(|x| PriceFeedId::from_id(&x.id))
            .collect())
    }
}

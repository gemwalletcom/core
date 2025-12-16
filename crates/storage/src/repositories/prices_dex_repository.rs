use crate::database::prices_dex::PricesDexStore;
use crate::models::{PriceDexAssetRow, PriceDexProviderRow, PriceDexRow};
use crate::{DatabaseClient, DatabaseError};
use primitives::{PriceFeedId, PriceFeedProvider};

pub trait PricesDexRepository {
    fn add_prices_dex_providers(&mut self, values: Vec<PriceDexProviderRow>) -> Result<usize, DatabaseError>;
    fn add_prices_dex(&mut self, values: Vec<PriceDexRow>) -> Result<usize, DatabaseError>;
    fn set_prices_dex(&mut self, values: Vec<PriceDexRow>) -> Result<usize, DatabaseError>;
    fn set_prices_dex_assets(&mut self, values: Vec<PriceDexAssetRow>) -> Result<usize, DatabaseError>;
    fn get_prices_dex(&mut self) -> Result<Vec<PriceDexRow>, DatabaseError>;
    fn get_prices_dex_assets(&mut self) -> Result<Vec<PriceDexAssetRow>, DatabaseError>;
    fn get_feed_ids_by_provider(&mut self, provider: PriceFeedProvider) -> Result<Vec<PriceFeedId>, DatabaseError>;
}

impl PricesDexRepository for DatabaseClient {
    fn add_prices_dex_providers(&mut self, values: Vec<PriceDexProviderRow>) -> Result<usize, DatabaseError> {
        Ok(PricesDexStore::add_prices_dex_providers(self, values)?)
    }

    fn add_prices_dex(&mut self, values: Vec<PriceDexRow>) -> Result<usize, DatabaseError> {
        Ok(PricesDexStore::add_prices_dex(self, values)?)
    }

    fn set_prices_dex(&mut self, values: Vec<PriceDexRow>) -> Result<usize, DatabaseError> {
        Ok(PricesDexStore::set_prices_dex(self, values)?)
    }

    fn set_prices_dex_assets(&mut self, values: Vec<PriceDexAssetRow>) -> Result<usize, DatabaseError> {
        Ok(PricesDexStore::set_prices_dex_assets(self, values)?)
    }

    fn get_prices_dex(&mut self) -> Result<Vec<PriceDexRow>, DatabaseError> {
        Ok(PricesDexStore::get_prices_dex(self)?)
    }

    fn get_prices_dex_assets(&mut self) -> Result<Vec<PriceDexAssetRow>, DatabaseError> {
        Ok(PricesDexStore::get_prices_dex_assets(self)?)
    }

    fn get_feed_ids_by_provider(&mut self, provider: PriceFeedProvider) -> Result<Vec<PriceFeedId>, DatabaseError> {
        Ok(PricesDexStore::get_prices_dex_by_provider(self, provider)?
            .into_iter()
            .filter_map(|x| PriceFeedId::from_id(&x.id))
            .collect())
    }
}

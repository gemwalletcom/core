use std::error::Error;
use chrono::NaiveDateTime;

use crate::DatabaseClient;
use crate::database::fiat::FiatStore;

pub trait FiatRepository {
    fn add_fiat_assets(&mut self, values: Vec<crate::models::FiatAsset>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn add_fiat_providers(&mut self, values: Vec<crate::models::FiatProvider>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn add_fiat_providers_countries(&mut self, values: Vec<crate::models::FiatProviderCountry>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_fiat_providers_countries(&mut self) -> Result<Vec<primitives::FiatProviderCountry>, Box<dyn Error + Send + Sync>>;
    fn add_fiat_transaction(&mut self, transaction: crate::models::FiatTransaction) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_fiat_assets(&mut self) -> Result<Vec<crate::models::FiatAsset>, Box<dyn Error + Send + Sync>>;
    fn get_fiat_assets_popular(&mut self, from: NaiveDateTime, limit: i64) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
    fn get_fiat_assets_for_asset_id(&mut self, asset_id: &str) -> Result<Vec<crate::models::FiatAsset>, Box<dyn Error + Send + Sync>>;
    fn set_fiat_rates(&mut self, rates: Vec<crate::models::FiatRate>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_fiat_rates(&mut self) -> Result<Vec<primitives::FiatRate>, Box<dyn Error + Send + Sync>>;
    fn get_fiat_rate(&mut self, currency: &str) -> Result<primitives::FiatRate, Box<dyn Error + Send + Sync>>;
    fn get_fiat_providers(&mut self) -> Result<Vec<crate::models::FiatProvider>, Box<dyn Error + Send + Sync>>;
    fn get_fiat_assets_is_enabled(&mut self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
}

impl FiatRepository for DatabaseClient {
    fn add_fiat_assets(&mut self, values: Vec<crate::models::FiatAsset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::add_fiat_assets(self, values)?)
    }

    fn add_fiat_providers(&mut self, values: Vec<crate::models::FiatProvider>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::add_fiat_providers(self, values)?)
    }

    fn add_fiat_providers_countries(&mut self, values: Vec<crate::models::FiatProviderCountry>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::add_fiat_providers_countries(self, values)?)
    }

    fn get_fiat_providers_countries(&mut self) -> Result<Vec<primitives::FiatProviderCountry>, Box<dyn Error + Send + Sync>> {
        let result = FiatStore::get_fiat_providers_countries(self)?;
        Ok(result.into_iter().map(|x| x.as_primitive()).collect())
    }

    fn add_fiat_transaction(&mut self, transaction: crate::models::FiatTransaction) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::add_fiat_transaction(self, transaction)?)
    }

    fn get_fiat_assets(&mut self) -> Result<Vec<crate::models::FiatAsset>, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::get_fiat_assets(self)?)
    }

    fn get_fiat_assets_popular(&mut self, from: NaiveDateTime, limit: i64) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::get_fiat_assets_popular(self, from, limit)?)
    }

    fn get_fiat_assets_for_asset_id(&mut self, asset_id: &str) -> Result<Vec<crate::models::FiatAsset>, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::get_fiat_assets_for_asset_id(self, asset_id)?)
    }

    fn set_fiat_rates(&mut self, rates: Vec<crate::models::FiatRate>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::set_fiat_rates(self, rates)?)
    }

    fn get_fiat_rates(&mut self) -> Result<Vec<primitives::FiatRate>, Box<dyn Error + Send + Sync>> {
        let result = FiatStore::get_fiat_rates(self)?;
        Ok(result.into_iter().map(|x| x.as_primitive()).collect())
    }

    fn get_fiat_rate(&mut self, currency: &str) -> Result<primitives::FiatRate, Box<dyn Error + Send + Sync>> {
        let result = FiatStore::get_fiat_rate(self, currency)?;
        Ok(result.as_primitive())
    }

    fn get_fiat_providers(&mut self) -> Result<Vec<crate::models::FiatProvider>, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::get_fiat_providers(self)?)
    }

    fn get_fiat_assets_is_enabled(&mut self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(FiatStore::get_fiat_assets_is_enabled(self)?)
    }
}
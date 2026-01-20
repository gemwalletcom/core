use crate::DatabaseError;
use chrono::NaiveDateTime;
use primitives::{FiatProviderCountry, FiatRate, FiatTransaction};

use crate::DatabaseClient;
use crate::database::fiat::FiatStore;

pub trait FiatRepository {
    fn add_fiat_assets(&mut self, values: Vec<crate::models::FiatAssetRow>) -> Result<usize, DatabaseError>;
    fn add_fiat_providers(&mut self, values: Vec<crate::models::FiatProviderRow>) -> Result<usize, DatabaseError>;
    fn add_fiat_providers_countries(&mut self, values: Vec<crate::models::FiatProviderCountryRow>) -> Result<usize, DatabaseError>;
    fn get_fiat_providers_countries(&mut self) -> Result<Vec<FiatProviderCountry>, DatabaseError>;
    fn add_fiat_transaction(&mut self, transaction: FiatTransaction) -> Result<usize, DatabaseError>;
    fn get_fiat_assets(&mut self) -> Result<Vec<crate::models::FiatAssetRow>, DatabaseError>;
    fn get_fiat_assets_popular(&mut self, from: NaiveDateTime, limit: i64) -> Result<Vec<String>, DatabaseError>;
    fn get_fiat_assets_for_asset_id(&mut self, asset_id: &str) -> Result<Vec<crate::models::FiatAssetRow>, DatabaseError>;
    fn set_fiat_rates(&mut self, rates: Vec<crate::models::FiatRateRow>) -> Result<usize, DatabaseError>;
    fn get_fiat_rates(&mut self) -> Result<Vec<FiatRate>, DatabaseError>;
    fn get_fiat_rate(&mut self, currency: &str) -> Result<FiatRate, DatabaseError>;
    fn get_fiat_providers(&mut self) -> Result<Vec<crate::models::FiatProviderRow>, DatabaseError>;
    fn get_fiat_assets_is_enabled(&mut self) -> Result<Vec<String>, DatabaseError>;
    fn add_fiat_quotes(&mut self, quotes: Vec<crate::models::FiatQuoteRow>) -> Result<usize, DatabaseError>;
    fn get_fiat_quote(&mut self, quote_id: &str) -> Result<crate::models::FiatQuoteRow, DatabaseError>;
    fn add_fiat_quote_request(&mut self, request: crate::models::FiatQuoteRequestRow) -> Result<usize, DatabaseError>;
}

impl FiatRepository for DatabaseClient {
    fn add_fiat_assets(&mut self, values: Vec<crate::models::FiatAssetRow>) -> Result<usize, DatabaseError> {
        Ok(FiatStore::add_fiat_assets(self, values)?)
    }

    fn add_fiat_providers(&mut self, values: Vec<crate::models::FiatProviderRow>) -> Result<usize, DatabaseError> {
        Ok(FiatStore::add_fiat_providers(self, values)?)
    }

    fn add_fiat_providers_countries(&mut self, values: Vec<crate::models::FiatProviderCountryRow>) -> Result<usize, DatabaseError> {
        Ok(FiatStore::add_fiat_providers_countries(self, values)?)
    }

    fn get_fiat_providers_countries(&mut self) -> Result<Vec<FiatProviderCountry>, DatabaseError> {
        let result = FiatStore::get_fiat_providers_countries(self)?;
        Ok(result.into_iter().map(|x| x.as_primitive()).collect())
    }

    fn add_fiat_transaction(&mut self, transaction: FiatTransaction) -> Result<usize, DatabaseError> {
        Ok(FiatStore::add_fiat_transaction(self, crate::models::FiatTransactionRow::from_primitive(transaction))?)
    }

    fn get_fiat_assets(&mut self) -> Result<Vec<crate::models::FiatAssetRow>, DatabaseError> {
        Ok(FiatStore::get_fiat_assets(self)?)
    }

    fn get_fiat_assets_popular(&mut self, from: NaiveDateTime, limit: i64) -> Result<Vec<String>, DatabaseError> {
        Ok(FiatStore::get_fiat_assets_popular(self, from, limit)?)
    }

    fn get_fiat_assets_for_asset_id(&mut self, asset_id: &str) -> Result<Vec<crate::models::FiatAssetRow>, DatabaseError> {
        Ok(FiatStore::get_fiat_assets_for_asset_id(self, asset_id)?)
    }

    fn set_fiat_rates(&mut self, rates: Vec<crate::models::FiatRateRow>) -> Result<usize, DatabaseError> {
        Ok(FiatStore::set_fiat_rates(self, rates)?)
    }

    fn get_fiat_rates(&mut self) -> Result<Vec<FiatRate>, DatabaseError> {
        let result = FiatStore::get_fiat_rates(self)?;
        Ok(result.into_iter().map(|x| x.as_primitive()).collect())
    }

    fn get_fiat_rate(&mut self, currency: &str) -> Result<FiatRate, DatabaseError> {
        let result = FiatStore::get_fiat_rate(self, currency)?;
        Ok(result.as_primitive())
    }

    fn get_fiat_providers(&mut self) -> Result<Vec<crate::models::FiatProviderRow>, DatabaseError> {
        Ok(FiatStore::get_fiat_providers(self)?)
    }

    fn get_fiat_assets_is_enabled(&mut self) -> Result<Vec<String>, DatabaseError> {
        Ok(FiatStore::get_fiat_assets_is_enabled(self)?)
    }

    fn add_fiat_quotes(&mut self, quotes: Vec<crate::models::FiatQuoteRow>) -> Result<usize, DatabaseError> {
        Ok(FiatStore::add_fiat_quotes(self, quotes)?)
    }

    fn get_fiat_quote(&mut self, quote_id: &str) -> Result<crate::models::FiatQuoteRow, DatabaseError> {
        Ok(FiatStore::get_fiat_quote(self, quote_id)?)
    }

    fn add_fiat_quote_request(&mut self, request: crate::models::FiatQuoteRequestRow) -> Result<usize, DatabaseError> {
        Ok(FiatStore::add_fiat_quote_request(self, request)?)
    }
}

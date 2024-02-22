use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use crate::model::{FiatClient, FiatMapping, FiatMappingMap, FiatRates};
use futures::future::join_all;
use primitives::{
    fiat_assets::FiatAssets, fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest,
};
//use futures::future::join_all;
use reqwest::Client as RequestClient;
use storage::DatabaseClient;

pub struct Client {
    database: DatabaseClient,
    providers: Vec<Box<dyn FiatClient + Send + Sync>>,
}

impl Client {
    pub async fn new(
        database_url: &str,
        providers: Vec<Box<dyn FiatClient + Send + Sync>>,
    ) -> Self {
        let database = DatabaseClient::new(database_url);

        Self {
            database,
            providers,
        }
    }

    pub fn request_client(timeout_seconds: u64) -> RequestClient {
        RequestClient::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .unwrap()
    }

    pub async fn get_assets(&mut self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self
            .database
            .get_fiat_assets()?
            .into_iter()
            .map(|x| x.asset_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let version = self.database.get_fiat_assets_version()?;

        Ok(FiatAssets {
            version: version as u32,
            asset_ids: assets,
        })
    }

    pub async fn get_fiat_rates(&mut self) -> Result<FiatRates, Box<dyn Error>> {
        let rates = self.database.get_fiat_rates()?;
        Ok(FiatRates { rates })
    }

    fn get_fiat_mapping(
        &mut self,
        asset_id: &str,
    ) -> Result<FiatMappingMap, Box<dyn Error + Send + Sync>> {
        let list = self.database.get_fiat_assets_for_asset_id(asset_id)?;
        let map: FiatMappingMap = list
            .into_iter()
            .map(|x| {
                (
                    x.provider,
                    FiatMapping {
                        symbol: x.symbol,
                        network: x.network,
                    },
                )
            })
            .collect();
        Ok(map)
    }

    pub async fn get_quotes(
        &mut self,
        request: FiatBuyRequest,
    ) -> Result<Vec<FiatQuote>, Box<dyn Error + Send + Sync>> {
        let fiat_mapping_map = self.get_fiat_mapping(&request.asset_id)?;
        let mut futures = vec![];

        for provider in &self.providers {
            if let Some(fiat_mapping) = fiat_mapping_map.get(provider.name().id().as_str()) {
                futures.push(provider.get_quote(request.clone(), fiat_mapping.clone()));
            }
        }

        let mut results: Vec<FiatQuote> = join_all(futures)
            .await
            .into_iter()
            .flatten()
            .map(|quote| {
                let mut result = quote.clone();
                result.crypto_amount = precision(quote.crypto_amount, 5);
                result
            })
            .collect();

        results.sort_by(|a, b| b.crypto_amount.partial_cmp(&a.crypto_amount).unwrap());

        Ok(results)
    }
}

#[allow(dead_code)]
fn precision(val: f64, precision: usize) -> f64 {
    format!("{:.prec$}", val, prec = precision)
        .parse::<f64>()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision() {
        assert_eq!(precision(1.123, 2), 1.12);
        assert_eq!(precision(1.123, 5), 1.123);
    }
}

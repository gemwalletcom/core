use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use crate::mercuryo::MercuryoClient;
use crate::model::{FiatClient, FiatMapping, FiatMappingMap, FiatRates};
use crate::moonpay::MoonPayClient;
use crate::ramp::RampClient;
use crate::transak::TransakClient;
use futures::future::join_all;
use primitives::{
    fiat_assets::FiatAssets, fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest,
};
//use futures::future::join_all;
use reqwest::Client as RequestClient;
use storage::DatabaseClient;

pub struct Client {
    database: DatabaseClient,
    transak: TransakClient,
    moonpay: MoonPayClient,
    #[allow(dead_code)]
    mercuryo: MercuryoClient,
    #[allow(dead_code)]
    ramp: RampClient,
}

impl Client {
    pub async fn new(
        database_url: &str,
        transak: TransakClient,
        moonpay: MoonPayClient,
        mercuryo: MercuryoClient,
        ramp: RampClient,
    ) -> Self {
        let database = DatabaseClient::new(database_url);

        Self {
            database,
            transak,
            moonpay,
            mercuryo,
            ramp,
        }
    }

    pub fn request_client(timeout_seconds: u64) -> RequestClient {
        RequestClient::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .unwrap()
    }

    pub async fn get_assets(&mut self) -> Result<FiatAssets, Box<dyn Error>> {
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

    pub fn get_fiat_mapping(&mut self, asset_id: &str) -> Result<FiatMappingMap, Box<dyn Error>> {
        let list = self.database.get_fiat_assets_for_asset_id(asset_id)?;
        let mut map: FiatMappingMap = FiatMappingMap::new();
        list.into_iter().for_each(|x: storage::models::FiatAsset| {
            map.insert(
                x.provider,
                FiatMapping {
                    symbol: x.symbol,
                    network: x.network,
                },
            );
        });
        Ok(map)
    }

    pub async fn get_quotes(
        &mut self,
        request: FiatBuyRequest,
        fiat_mapping_map: FiatMappingMap,
    ) -> Result<Vec<FiatQuote>, Box<dyn Error + Send + Sync>> {
        let mut futures = vec![];

        if let Some(value) = fiat_mapping_map.get(self.ramp.name().id().as_str()) {
            futures.push(self.ramp.get_quote(request.clone(), value.clone()));
        }

        if let Some(value) = fiat_mapping_map.get(self.moonpay.name().id().as_str()) {
            futures.push(self.moonpay.get_quote(request.clone(), value.clone()));
        }

        if let Some(value) = fiat_mapping_map.get(self.transak.name().id().as_str()) {
            futures.push(self.transak.get_quote(request.clone(), value.clone()));
        }

        if let Some(value) = fiat_mapping_map.get(self.mercuryo.name().id().as_str()) {
            futures.push(self.mercuryo.get_quote(request.clone(), value.clone()));
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

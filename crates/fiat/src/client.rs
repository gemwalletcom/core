use std::error::Error;
use std::time::Duration;

use crate::{
    model::{FiatMapping, FiatMappingMap},
    FiatProvider,
};
use futures::future::join_all;
use primitives::{
    fiat_assets::FiatAssets,
    fiat_quote::{FiatQuote, FiatQuoteError},
    fiat_quote_request::FiatBuyRequest,
};
use primitives::{fiat_quote::FiatQuotes, fiat_quote_request::FiatSellRequest};
use reqwest::Client as RequestClient;
use storage::DatabaseClient;

pub struct FiatClient {
    database: DatabaseClient,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
}

impl FiatClient {
    pub async fn new(database_url: &str, providers: Vec<Box<dyn FiatProvider + Send + Sync>>) -> Self {
        let database = DatabaseClient::new(database_url);

        Self { database, providers }
    }

    pub fn request_client(timeout_seconds: u64) -> RequestClient {
        RequestClient::builder().timeout(Duration::from_secs(timeout_seconds)).build().unwrap()
    }

    pub async fn get_on_ramp_assets(&mut self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self.database.get_assets_is_buyable()?;
        Ok(FiatAssets {
            version: assets.clone().len() as u32,
            asset_ids: assets,
        })
    }

    pub async fn get_off_ramp_assets(&mut self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self.database.get_assets_is_sellable()?;
        Ok(FiatAssets {
            version: assets.clone().len() as u32,
            asset_ids: assets,
        })
    }

    pub async fn create_fiat_webhook(&mut self, provider_name: &str, data: serde_json::Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        for provider in &self.providers {
            if provider.name().id() == provider_name {
                let transaction = provider.webhook(data).await?;
                let transaction = storage::models::FiatTransaction::from_primitive(transaction.clone());

                let _ = self.database.add_fiat_transaction(transaction)?;

                return Ok(true);
            }
        }
        Ok(false)
    }

    fn get_fiat_mapping(&mut self, asset_id: &str) -> Result<FiatMappingMap, Box<dyn Error + Send + Sync>> {
        let list = self
            .database
            .get_fiat_assets_for_asset_id(asset_id)?
            .into_iter()
            .filter(|x| x.is_enabled())
            .collect::<Vec<_>>();

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

    pub async fn get_buy_quotes(&mut self, request: FiatBuyRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        let fiat_mapping_map = self.get_fiat_mapping(&request.asset_id)?;
        let mut quotes = vec![];
        let mut errors = vec![];

        let futures = self
            .providers
            .iter()
            .filter_map(|provider| {
                fiat_mapping_map.get(provider.name().id().as_str()).and_then(|fiat_mapping| {
                    if let Some(provider_id) = request.provider_id.clone() {
                        // If provider_name is provided, only get quote from that provider
                        if provider_id == provider.name().id() {
                            return Some(provider.get_buy_quote(request.clone(), fiat_mapping.clone()));
                        }
                        None
                    } else {
                        Some(provider.get_buy_quote(request.clone(), fiat_mapping.clone()))
                    }
                })
            })
            .collect::<Vec<_>>();

        join_all(futures).await.into_iter().for_each(|x| match x {
            Ok(quote) => {
                quotes.push(quote);
            }
            Err(e) => {
                errors.push(FiatQuoteError {
                    provider: "".to_string(), //TODO: Add provider
                    error: e.to_string(),
                });
            }
        });

        quotes.sort_by(|a, b| b.crypto_amount.partial_cmp(&a.crypto_amount).unwrap());

        Ok(FiatQuotes { quotes, errors })
    }

    pub async fn get_sell_quotes(&mut self, request: FiatSellRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        let fiat_mapping_map = self.get_fiat_mapping(&request.asset_id)?;
        let mut futures = vec![];

        for provider in &self.providers {
            if let Some(fiat_mapping) = fiat_mapping_map.get(provider.name().id().as_str()) {
                futures.push(provider.get_sell_quote(request.clone(), fiat_mapping.clone()));
            }
        }

        let mut quotes: Vec<FiatQuote> = join_all(futures)
            .await
            .into_iter()
            .flatten()
            .map(|quote| {
                let mut result = quote.clone();
                result.crypto_amount = precision(quote.crypto_amount, 5);
                result
            })
            .collect();

        quotes.sort_by(|a, b| b.crypto_amount.partial_cmp(&a.crypto_amount).unwrap());

        Ok(FiatQuotes { quotes, errors: vec![] })
    }
}

#[allow(dead_code)]
fn precision(val: f64, precision: usize) -> f64 {
    format!("{:.prec$}", val, prec = precision).parse::<f64>().unwrap()
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

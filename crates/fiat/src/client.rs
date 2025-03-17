use std::error::Error;
use std::time::Duration;

use crate::{
    model::{FiatMapping, FiatMappingMap},
    FiatProvider,
};
use futures::future::join_all;
use primitives::{FiatAssets, FiatQuote, FiatQuoteError, FiatQuoteRequest, FiatQuotes};
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

    pub fn get_providers(&self, request: FiatQuoteRequest) -> Vec<&(dyn FiatProvider + Send + Sync)> {
        self.providers
            .iter()
            .filter(|x| request.provider_id.as_deref().map_or(true, |id| x.name().id() == id))
            .map(|x| x.as_ref())
            .collect()
    }

    pub async fn get_buy_quotes(&mut self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        self.get_quotes(
            request,
            |provider, request, mapping| provider.get_buy_quote(request, mapping),
            sort_by_crypto_amount,
        )
        .await
    }

    pub async fn get_sell_quotes(&mut self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        self.get_quotes(
            request,
            |provider, request, mapping| provider.get_sell_quote(request, mapping),
            sort_by_fiat_amount,
        )
        .await
    }

    async fn get_quotes<F>(
        &mut self,
        request: FiatQuoteRequest,
        quote_fn: F,
        sort_fn: fn(&FiatQuote, &FiatQuote) -> std::cmp::Ordering,
    ) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>>
    where
        F: Fn(&dyn FiatProvider, FiatQuoteRequest, FiatMapping) -> futures::future::BoxFuture<'_, Result<FiatQuote, Box<dyn Error + Send + Sync>>>
            + Send
            + Sync,
    {
        let fiat_mapping_map = self.get_fiat_mapping(&request.asset_id)?;

        let providers = self.get_providers(request.clone());
        let futures = providers.into_iter().filter_map(|provider| {
            let provider_id = provider.name().id().to_string();
            fiat_mapping_map.get(&provider_id).map(|mapping| {
                let quote_fn = &quote_fn;
                let request = request.clone();
                let mapping = mapping.clone();
                async move {
                    match quote_fn(provider, request, mapping).await {
                        Ok(quote) => Ok(quote),
                        Err(e) => Err(FiatQuoteError::new(provider_id, e.to_string())),
                    }
                }
            })
        });

        let results = join_all(futures).await;

        let mut quotes = Vec::new();
        let mut errors = Vec::new();

        for result in results {
            match result {
                Ok(quote) => quotes.push(quote),
                Err(e) => errors.push(e),
            }
        }

        for quote in &mut quotes {
            quote.crypto_amount = precision(quote.crypto_amount, 5);
        }

        quotes.sort_by(sort_fn);

        Ok(FiatQuotes { quotes, errors })
    }
}

#[allow(dead_code)]
fn precision(val: f64, precision: usize) -> f64 {
    format!("{:.prec$}", val, prec = precision).parse::<f64>().unwrap()
}

fn sort_by_crypto_amount(a: &FiatQuote, b: &FiatQuote) -> std::cmp::Ordering {
    b.crypto_amount.partial_cmp(&a.crypto_amount).unwrap()
}

fn sort_by_fiat_amount(a: &FiatQuote, b: &FiatQuote) -> std::cmp::Ordering {
    b.fiat_amount.partial_cmp(&a.fiat_amount).unwrap()
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

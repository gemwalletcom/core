use cacher::CacherClient;
use number_formatter::BigNumberFormatter;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

use crate::{
    error::FiatError,
    ip_check_client::IPAddressInfo,
    model::{FiatMapping, FiatMappingMap},
    FiatProvider, IPCheckClient,
};
use futures::future::join_all;
use primitives::{Asset, FiatAssets, FiatProviderCountry, FiatQuote, FiatQuoteError, FiatQuoteRequest, FiatQuoteType, FiatQuotes};
use reqwest::Client as RequestClient;
use storage::DatabaseClient;

pub struct FiatClient {
    database: DatabaseClient,
    cacher: CacherClient,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
    ip_check_client: IPCheckClient,
}

impl FiatClient {
    pub async fn new(database_url: &str, cacher: CacherClient, providers: Vec<Box<dyn FiatProvider + Send + Sync>>, ip_check_client: IPCheckClient) -> Self {
        let database = DatabaseClient::new(database_url);

        Self {
            database,
            cacher,
            providers,
            ip_check_client,
        }
    }

    pub fn request_client(timeout_seconds: u64) -> RequestClient {
        RequestClient::builder().timeout(Duration::from_secs(timeout_seconds)).build().unwrap()
    }

    pub async fn get_on_ramp_assets(&mut self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self.database.get_assets_is_buyable()?;
        Ok(FiatAssets {
            version: assets.clone().len() as u32,
            asset_ids: assets.into_iter().map(|x| x.id).collect::<Vec<String>>(),
        })
    }

    pub async fn get_fiat_providers_countries(&mut self) -> Result<Vec<FiatProviderCountry>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.get_fiat_providers_countries()?.into_iter().map(|x| x.as_primitive()).collect())
    }

    pub async fn get_off_ramp_assets(&mut self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self.database.get_assets_is_sellable()?;
        Ok(FiatAssets {
            version: assets.clone().len() as u32,
            asset_ids: assets.into_iter().map(|x| x.id).collect::<Vec<String>>(),
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
                    x.clone().provider,
                    FiatMapping {
                        symbol: x.clone().symbol,
                        network: x.clone().network,
                        unsupported_countries: x.clone().unsupported_countries(),
                    },
                )
            })
            .collect();
        Ok(map)
    }

    pub fn get_providers(&self, request: FiatQuoteRequest) -> Vec<&(dyn FiatProvider + Send + Sync)> {
        self.providers
            .iter()
            .filter(|x| request.provider_id.as_deref().is_none_or(|id| x.name().id() == id))
            .map(|x| x.as_ref())
            .collect()
    }

    pub async fn get_asset(&mut self, asset_id: &str) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Ok(self.database.get_asset(asset_id)?.as_primitive())
    }

    pub async fn get_quotes(&mut self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        let asset = self.database.get_asset(&request.asset_id)?.as_primitive();
        let fiat_providers_countries = self.get_fiat_providers_countries().await?;
        let ip_address_info = self.get_ip_address(&request.ip_address).await?;
        let fiat_mapping_map = self.get_fiat_mapping(&request.asset_id)?;

        let quotes = match request.clone().quote_type {
            FiatQuoteType::Buy => {
                let fiat_amount = request.clone().fiat_amount.unwrap();
                let fiat_value = BigNumberFormatter::f64_as_value(fiat_amount, asset.decimals as u32).unwrap_or_default();
                self.get_quotes_in_parallel(
                    request.clone(),
                    fiat_mapping_map,
                    ip_address_info.clone(),
                    fiat_providers_countries,
                    |provider, request, mapping| provider.get_buy_quote(request.get_buy_quote(asset.clone(), fiat_value.clone()), mapping),
                    sort_by_crypto_amount,
                )
                .await
            }
            FiatQuoteType::Sell => {
                let crypto_value = &request.clone().crypto_value.unwrap();
                let crypto_amount = BigNumberFormatter::value_as_f64(crypto_value, asset.decimals as u32).unwrap_or_default();

                self.get_quotes_in_parallel(
                    request.clone(),
                    fiat_mapping_map,
                    ip_address_info.clone(),
                    fiat_providers_countries,
                    |provider, request, mapping| provider.get_sell_quote(request.get_sell_quote(asset.clone(), crypto_amount), mapping),
                    sort_by_fiat_amount,
                )
                .await
            }
        }?;
        Ok(quotes)
    }

    pub async fn get_ip_address(&mut self, ip_address: &str) -> Result<IPAddressInfo, Box<dyn Error + Send + Sync>> {
        let key = format!("fiat_ip_resolver_ip_address:{ip_address}");
        self.cacher
            .get_or_set_value(&key, || self.ip_check_client.get_ip_address(ip_address), Some(86400))
            .await
    }

    async fn get_quotes_in_parallel<F>(
        &mut self,
        request: FiatQuoteRequest,
        fiat_mapping_map: HashMap<String, FiatMapping>,
        ip_address_info: IPAddressInfo,
        countries: Vec<FiatProviderCountry>,
        quote_fn: F,
        sort_fn: fn(&FiatQuote, &FiatQuote) -> std::cmp::Ordering,
    ) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>>
    where
        F: Fn(&dyn FiatProvider, FiatQuoteRequest, FiatMapping) -> futures::future::BoxFuture<'_, Result<FiatQuote, Box<dyn Error + Send + Sync>>>
            + Send
            + Sync,
    {
        let providers = self.get_providers(request.clone());
        let futures = providers.into_iter().filter_map(|provider| {
            let provider_id = provider.name().id().to_string();
            let countries = countries
                .iter()
                .filter(|x| x.provider == provider_id)
                .map(|x| x.alpha2.clone())
                .collect::<HashSet<_>>();

            fiat_mapping_map.get(&provider_id).map(|mapping| {
                let quote_fn = &quote_fn;
                let request = request.clone();
                let mapping = mapping.clone();
                let country_code = ip_address_info.clone().alpha2;

                async move {
                    if !countries.contains(&country_code) {
                        Err(FiatQuoteError::new(provider_id, FiatError::UnsupportedCountry(country_code).to_string()))
                    } else if mapping.unsupported_countries.clone().contains_key(&country_code) {
                        Err(FiatQuoteError::new(
                            provider_id,
                            FiatError::UnsupportedCountryAsset(country_code, mapping.symbol).to_string(),
                        ))
                    } else {
                        match quote_fn(provider, request, mapping).await {
                            Ok(quote) => Ok(quote),
                            Err(e) => Err(FiatQuoteError::new(provider_id, e.to_string())),
                        }
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
    format!("{val:.precision$}").parse::<f64>().unwrap()
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

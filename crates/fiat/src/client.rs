use cacher::CacherClient;
use number_formatter::BigNumberFormatter;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

use crate::{
    FiatConfig, FiatProvider, IPCheckClient,
    error::FiatQuoteError,
    ip_check_client::IPAddressInfo,
    model::{FiatMapping, FiatMappingMap},
};
use futures::future::join_all;
use primitives::{Asset, FiatAssets, FiatProviderCountry, FiatQuote, FiatQuoteError as PrimitiveFiatQuoteError, FiatQuoteRequest, FiatQuoteType, FiatQuotes};
use reqwest::Client as RequestClient;
use storage::{AssetFilter, Database};
use streamer::{FiatWebhookPayload, StreamProducer};

pub struct FiatClient {
    database: Database,
    cacher: CacherClient,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
    ip_check_client: IPCheckClient,
    stream_producer: StreamProducer,
    config: FiatConfig,
}

impl FiatClient {
    pub fn new(
        database: Database,
        cacher: CacherClient,
        providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
        ip_check_client: IPCheckClient,
        stream_producer: StreamProducer,
        config: FiatConfig,
    ) -> Self {
        Self {
            database,
            cacher,
            providers,
            ip_check_client,
            stream_producer,
            config,
        }
    }

    fn provider(&self, provider_name: &str) -> Result<&(dyn FiatProvider + Send + Sync), Box<dyn std::error::Error + Send + Sync>> {
        self.providers
            .iter()
            .find(|provider| provider.name().id() == provider_name)
            .map(|provider| provider.as_ref())
            .ok_or_else(|| format!("Provider {} not found", provider_name).into())
    }

    pub fn request_client(timeout: Duration) -> RequestClient {
        RequestClient::builder().timeout(timeout).build().unwrap()
    }

    pub async fn get_on_ramp_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self.database.client()?.assets().get_assets_by_filter(vec![AssetFilter::IsBuyable(true)])?;
        Ok(FiatAssets {
            version: assets.clone().len() as u32,
            asset_ids: assets.into_iter().map(|x| x.asset.id.to_string()).collect::<Vec<String>>(),
        })
    }

    pub async fn get_off_ramp_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self.database.client()?.assets().get_assets_by_filter(vec![AssetFilter::IsSellable(true)])?;
        Ok(FiatAssets {
            version: assets.clone().len() as u32,
            asset_ids: assets.into_iter().map(|x| x.asset.id.to_string()).collect::<Vec<String>>(),
        })
    }

    pub async fn get_fiat_providers_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.fiat().get_fiat_providers_countries()?)
    }

    pub async fn get_order_status(&self, provider_name: &str, order_id: &str) -> Result<primitives::FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        self.provider(provider_name)?.get_order_status(order_id).await
    }

    pub async fn process_and_publish_webhook(
        &self,
        provider_name: &str,
        webhook_data: serde_json::Value,
    ) -> Result<FiatWebhookPayload, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider(provider_name)?;
        let webhook = match provider.process_webhook(webhook_data.clone()).await {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to decode webhook payload: {}, JSON payload: {}", e, webhook_data);
                return Err(e);
            }
        };
        let payload = FiatWebhookPayload::new(provider.name(), webhook_data.clone(), webhook.clone());
        match webhook {
            streamer::FiatWebhook::OrderId(_) | streamer::FiatWebhook::Transaction(_) => {
                self.stream_producer.publish(streamer::QueueName::FiatOrderWebhooks, &payload).await?;
            }
            streamer::FiatWebhook::None => {}
        }
        Ok(payload)
    }

    fn get_fiat_mapping(&self, asset_id: &str) -> Result<FiatMappingMap, Box<dyn Error + Send + Sync>> {
        let list = self
            .database
            .client()?
            .fiat()
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
                        buy_limits: x.clone().buy_limits(),
                        sell_limits: x.clone().sell_limits(),
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

    pub async fn get_asset(&self, asset_id: &str) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.assets().get_asset(asset_id)?)
    }

    pub async fn get_quotes(&self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        let asset = self.database.client()?.assets().get_asset(&request.asset_id)?;

        if self.config.validate_subscription {
            let is_subscribed = self.is_address_subscribed(&asset, &request.wallet_address)?;
            if !is_subscribed {
                let error = FiatQuoteError::AddressNotSubscribed(request.wallet_address.to_string());
                return Ok(FiatQuotes::new_error(PrimitiveFiatQuoteError::new(None, error.to_string())));
            }
        }

        let fiat_providers_countries = self.get_fiat_providers_countries().await?;
        let ip_address_info = match self.get_ip_address(&request.ip_address).await {
            Ok(info) => info,
            Err(e) => {
                let error = FiatQuoteError::IpAddressValidationFailed(e.to_string());
                return Ok(FiatQuotes::new_error(PrimitiveFiatQuoteError::new(None, error.to_string())));
            }
        };
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

    pub async fn get_ip_address(&self, ip_address: &str) -> Result<IPAddressInfo, Box<dyn Error + Send + Sync>> {
        let key = format!("fiat_ip_resolver_ip_address:{ip_address}");
        self.cacher
            .get_or_set_value(&key, || self.ip_check_client.get_ip_address(ip_address), Some(86400))
            .await
    }

    fn is_address_subscribed(&self, asset: &Asset, wallet_address: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .client()?
            .subscriptions()
            .get_subscription_address_exists(asset.chain, wallet_address)?)
    }

    fn check_asset_limits(request: &FiatQuoteRequest, mapping: &FiatMapping) -> Result<(), FiatQuoteError> {
        let fiat_currency = request.fiat_currency.clone();

        let limits = match request.quote_type {
            FiatQuoteType::Buy => &mapping.buy_limits,
            FiatQuoteType::Sell => &mapping.sell_limits,
        };

        if limits.is_empty() {
            return Ok(());
        }

        let amount = match request.quote_type {
            FiatQuoteType::Buy => request.fiat_amount,
            FiatQuoteType::Sell => request.fiat_amount, // For sell, we'd need fiat equivalent
        };

        if let Some(amount) = amount {
            for limit in limits {
                if limit.currency == fiat_currency {
                    if let Some(min_amount) = limit.min_amount
                        && amount < min_amount
                    {
                        return Err(FiatQuoteError::InsufficientAmount(amount, min_amount));
                    }
                    if let Some(max_amount) = limit.max_amount
                        && amount > max_amount
                    {
                        return Err(FiatQuoteError::ExcessiveAmount(amount, max_amount));
                    }
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    async fn get_quotes_in_parallel<F>(
        &self,
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
                        Err(PrimitiveFiatQuoteError::new(
                            Some(provider_id.clone()),
                            FiatQuoteError::UnsupportedCountry(country_code).to_string(),
                        ))
                    } else if mapping.unsupported_countries.clone().contains_key(&country_code) {
                        Err(PrimitiveFiatQuoteError::new(
                            Some(provider_id.clone()),
                            FiatQuoteError::UnsupportedCountryAsset(country_code, mapping.symbol.clone()).to_string(),
                        ))
                    } else {
                        match Self::check_asset_limits(&request, &mapping) {
                            Ok(_) => match quote_fn(provider, request, mapping).await {
                                Ok(quote) => Ok(quote),
                                Err(e) => Err(PrimitiveFiatQuoteError::new(Some(provider_id), e.to_string())),
                            },
                            Err(limit_error) => Err(PrimitiveFiatQuoteError::new(Some(provider_id), limit_error.to_string())),
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
    use primitives::FiatQuoteType;
    use primitives::fiat_assets::FiatAssetLimits;
    use std::collections::HashMap;

    #[test]
    fn test_precision() {
        assert_eq!(precision(1.123, 2), 1.12);
        assert_eq!(precision(1.123, 5), 1.123);
    }

    #[test]
    fn check_asset_limits_buy_within_limits() {
        let request = FiatQuoteRequest::mock();
        let mapping = FiatMapping {
            symbol: "BTC".to_string(),
            network: None,
            unsupported_countries: HashMap::new(),
            buy_limits: vec![FiatAssetLimits::mock_usd(50.0, 200.0)],
            sell_limits: vec![],
        };
        assert!(FiatClient::check_asset_limits(&request, &mapping).is_ok());
    }

    #[test]
    fn check_asset_limits_buy_below_minimum() {
        let mut request = FiatQuoteRequest::mock();
        request.fiat_amount = Some(25.0);
        let mapping = FiatMapping {
            symbol: "BTC".to_string(),
            network: None,
            unsupported_countries: HashMap::new(),
            buy_limits: vec![FiatAssetLimits::mock_usd(50.0, 200.0)],
            sell_limits: vec![],
        };
        match FiatClient::check_asset_limits(&request, &mapping).unwrap_err() {
            FiatQuoteError::InsufficientAmount(amount, min) => {
                assert_eq!(amount, 25.0);
                assert_eq!(min, 50.0);
            }
            _ => panic!("Expected InsufficientAmount error"),
        }
    }

    #[test]
    fn check_asset_limits_buy_above_maximum() {
        let mut request = FiatQuoteRequest::mock();
        request.fiat_amount = Some(300.0);
        let mapping = FiatMapping {
            symbol: "BTC".to_string(),
            network: None,
            unsupported_countries: HashMap::new(),
            buy_limits: vec![FiatAssetLimits::mock_usd(50.0, 200.0)],
            sell_limits: vec![],
        };
        match FiatClient::check_asset_limits(&request, &mapping).unwrap_err() {
            FiatQuoteError::ExcessiveAmount(amount, max) => {
                assert_eq!(amount, 300.0);
                assert_eq!(max, 200.0);
            }
            _ => panic!("Expected ExcessiveAmount error"),
        }
    }

    #[test]
    fn check_asset_limits_sell_within_limits() {
        let mut request = FiatQuoteRequest::mock();
        request.quote_type = FiatQuoteType::Sell;
        let mapping = FiatMapping {
            symbol: "BTC".to_string(),
            network: None,
            unsupported_countries: HashMap::new(),
            buy_limits: vec![],
            sell_limits: vec![FiatAssetLimits::mock_usd(50.0, 200.0)],
        };
        assert!(FiatClient::check_asset_limits(&request, &mapping).is_ok());
    }

    #[test]
    fn check_asset_limits_no_limits() {
        let request = FiatQuoteRequest::mock();
        let mapping = FiatMapping {
            symbol: "BTC".to_string(),
            network: None,
            unsupported_countries: HashMap::new(),
            buy_limits: vec![],
            sell_limits: vec![],
        };
        assert!(FiatClient::check_asset_limits(&request, &mapping).is_ok());
    }
}

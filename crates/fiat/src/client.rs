use cacher::CacherClient;
use number_formatter::BigNumberFormatter;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

use crate::{
    CachedFiatQuoteData, FiatCacherClient, FiatConfig, FiatProvider, IPCheckClient,
    error::FiatQuoteError,
    ip_check_client::IPAddressInfo,
    model::{FiatMapping, FiatMappingMap},
};
use futures::future::join_all;
use primitives::{
    Asset, FiatAssets, FiatProvider as PrimitiveFiatProvider, FiatProviderCountry, FiatQuote, FiatQuoteError as PrimitiveFiatQuoteError, FiatQuoteOldRequest,
    FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlData, FiatQuotes, FiatQuotesOld,
};
use reqwest::Client as RequestClient;
use storage::{AssetFilter, Database};
use streamer::{FiatWebhookPayload, StreamProducer};

pub struct FiatClient {
    database: Database,
    cacher: CacherClient,
    fiat_cacher: FiatCacherClient,
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
            cacher: cacher.clone(),
            fiat_cacher: FiatCacherClient::new(cacher),
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

    fn get_fiat_mapping(&self, asset_id: &str) -> Result<(FiatMappingMap, Vec<PrimitiveFiatProvider>), Box<dyn Error + Send + Sync>> {
        let mut db = self.database.client()?;
        let fiat_assets = db.fiat().get_fiat_assets_for_asset_id(asset_id)?;
        let providers = db.fiat().get_fiat_providers()?.into_iter().map(|p| p.as_primitive()).collect();

        let map: FiatMappingMap = fiat_assets
            .into_iter()
            .filter(|x| x.is_enabled())
            .map(|x| {
                (
                    x.clone().provider,
                    FiatMapping {
                        asset_symbol: primitives::FiatAssetSymbol {
                            symbol: x.clone().symbol,
                            network: x.clone().network,
                        },
                        unsupported_countries: x.clone().unsupported_countries(),
                        buy_limits: x.clone().buy_limits(),
                        sell_limits: x.clone().sell_limits(),
                    },
                )
            })
            .collect();
        Ok((map, providers))
    }

    pub fn get_providers(&self, provider_id: Option<String>) -> Vec<&(dyn FiatProvider + Send + Sync)> {
        self.providers
            .iter()
            .filter(|x| provider_id.as_deref().is_none_or(|id| x.name().id() == id))
            .map(|x| x.as_ref())
            .collect()
    }

    pub fn get_providers_for_request(&self, request: &FiatQuoteOldRequest) -> Vec<&(dyn FiatProvider + Send + Sync)> {
        self.get_providers(request.provider_id.clone())
    }

    pub async fn get_asset(&self, asset_id: &str) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.assets().get_asset(asset_id)?)
    }

    pub async fn get_quotes_old(&self, request: FiatQuoteOldRequest) -> Result<FiatQuotesOld, Box<dyn Error + Send + Sync>> {
        let asset = self.database.client()?.assets().get_asset(&request.asset_id)?;

        if self.config.validate_subscription {
            let is_subscribed = self.is_address_subscribed(&asset, &request.wallet_address)?;
            if !is_subscribed {
                let error = FiatQuoteError::AddressNotSubscribed(request.wallet_address.to_string());
                return Ok(FiatQuotesOld::new_error(PrimitiveFiatQuoteError::new(None, error.to_string())));
            }
        }

        let fiat_providers_countries = self.get_fiat_providers_countries().await?;
        let ip_address_info = match self.get_ip_address(&request.ip_address).await {
            Ok(info) => info,
            Err(e) => {
                let error = FiatQuoteError::IpAddressValidationFailed(e.to_string());
                return Ok(FiatQuotesOld::new_error(PrimitiveFiatQuoteError::new(None, error.to_string())));
            }
        };
        let (fiat_mapping_map, providers) = self.get_fiat_mapping(&request.asset_id)?;

        let quotes = match request.clone().quote_type {
            FiatQuoteType::Buy => {
                let fiat_amount = request.clone().fiat_amount.unwrap();
                let fiat_value = BigNumberFormatter::f64_as_value(fiat_amount, asset.decimals as u32).unwrap_or_default();
                self.get_quotes_in_parallel(
                    request.clone(),
                    fiat_mapping_map,
                    ip_address_info.clone(),
                    fiat_providers_countries.clone(),
                    &providers,
                    |provider, request, mapping| provider.get_buy_quote_old(request.get_buy_quote(asset.clone(), fiat_value.clone()), mapping),
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
                    &providers,
                    |provider, request, mapping| provider.get_sell_quote_old(request.get_sell_quote(asset.clone(), crypto_amount), mapping),
                    sort_by_fiat_amount,
                )
                .await
            }
        }?;
        Ok(quotes)
    }

    pub async fn get_quotes(&self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        let _asset = self.database.client()?.assets().get_asset(&request.asset_id)?;

        let fiat_providers_countries = self.get_fiat_providers_countries().await?;
        let ip_address_info = match self.get_ip_address(&request.ip_address).await {
            Ok(info) => info,
            Err(e) => {
                return Err(format!("IP address validation failed: {}", e).into());
            }
        };
        let (fiat_mapping_map, db_providers) = self.get_fiat_mapping(&request.asset_id)?;

        let provider_impls = self.get_providers(request.provider_id.clone());

        let futures = provider_impls.into_iter().filter_map(|provider| {
            let provider_id = provider.name().id().to_string();

            let db_provider = db_providers.iter().find(|p| p.id == provider_id)?;
            let is_enabled = match request.quote_type {
                FiatQuoteType::Buy => db_provider.is_buy_enabled(),
                FiatQuoteType::Sell => db_provider.is_sell_enabled(),
            };

            if !is_enabled {
                return None;
            }

            let countries = fiat_providers_countries
                .iter()
                .filter(|x| x.provider == provider_id)
                .map(|x| x.alpha2.clone())
                .collect::<HashSet<_>>();

            fiat_mapping_map.get(&provider_id).map(|mapping| {
                let request = request.clone();
                let mapping = mapping.clone();
                let country_code = ip_address_info.clone().alpha2;
                let provider_id_clone = provider_id.clone();

                async move {
                    if !countries.contains(&country_code) {
                        return Err((
                            provider_id_clone.clone(),
                            Box::<dyn Error + Send + Sync>::from(format!("Unsupported country: {}", country_code)),
                        ));
                    }
                    if mapping.unsupported_countries.clone().contains_key(&country_code) {
                        return Err((
                            provider_id_clone.clone(),
                            Box::<dyn Error + Send + Sync>::from(format!("Unsupported country for asset: {}", country_code)),
                        ));
                    }

                    let quote_request = FiatQuoteRequest {
                        asset_id: request.asset_id.clone(),
                        quote_type: request.quote_type.clone(),
                        currency: request.currency.clone(),
                        amount: request.amount,
                        provider_id: request.provider_id.clone(),
                        ip_address: request.ip_address.clone(),
                    };

                    let start = std::time::Instant::now();
                    let response = match request.quote_type {
                        FiatQuoteType::Buy => provider.get_quote_buy(quote_request.clone(), mapping.clone()).await,
                        FiatQuoteType::Sell => provider.get_quote_sell(quote_request.clone(), mapping.clone()).await,
                    }
                    .map_err(|e| (provider_id_clone.clone(), e))?;
                    let latency = start.elapsed().as_millis() as u64;
                    let quote = FiatQuote::new(
                        response.quote_id,
                        request.asset_id.clone(),
                        provider.name().as_fiat_provider(),
                        quote_request.quote_type,
                        response.fiat_amount,
                        quote_request.currency,
                        response.crypto_amount,
                        latency,
                    );
                    Ok((
                        provider_id_clone,
                        CachedFiatQuoteData {
                            quote,
                            asset_symbol: mapping.asset_symbol,
                        },
                    ))
                }
            })
        });

        let results = join_all(futures).await;

        let mut quotes = Vec::new();
        let mut errors = Vec::new();

        for result in results {
            match result {
                Ok((_, cached_quote)) => quotes.push(cached_quote),
                Err((provider_id, e)) => errors.push(primitives::FiatQuoteError::new(Some(provider_id), e.to_string())),
            }
        }

        self.fiat_cacher.set_quotes(&quotes).await?;

        let quotes = quotes.into_iter().map(|x| x.quote).collect();
        Ok(FiatQuotes { quotes, errors })
    }

    pub async fn get_quote_url(
        &self,
        quote_id: &str,
        wallet_address: &str,
        ip_address: &str,
        device_id: &str,
    ) -> Result<(FiatQuoteUrl, FiatQuote), Box<dyn Error + Send + Sync>> {
        let device = self.database.client()?.get_device(device_id)?;

        let quote = self.fiat_cacher.get_quote(quote_id).await?;
        let provider = self.provider(&quote.quote.provider.id)?;

        let data = FiatQuoteUrlData {
            quote: quote.quote.clone(),
            asset_symbol: quote.asset_symbol,
            wallet_address: wallet_address.to_string(),
            ip_address: ip_address.to_string(),
            locale: device.locale,
        };

        let url = provider.get_quote_url(data).await?;

        let db_quote = storage::models::FiatQuote::from_primitive(&quote.quote);
        self.database.client()?.add_fiat_quotes(vec![db_quote])?;

        self.database.client()?.add_fiat_quote_request(storage::models::FiatQuoteRequest {
            device_id: device.id,
            quote_id: quote_id.to_string(),
        })?;

        Ok((url, quote.quote))
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

    fn check_asset_limits_old(request: &FiatQuoteOldRequest, mapping: &FiatMapping) -> Result<(), FiatQuoteError> {
        let fiat_currency = request.fiat_currency.clone();

        let limits = match request.quote_type {
            FiatQuoteType::Buy => &mapping.buy_limits,
            FiatQuoteType::Sell => &mapping.sell_limits,
        };

        if limits.is_empty() {
            return Ok(());
        }

        let amount = match request.quote_type {
            FiatQuoteType::Buy => request.fiat_amount.unwrap_or(0.0),
            FiatQuoteType::Sell => request.fiat_amount.unwrap_or(0.0),
        };

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

        Ok(())
    }

    async fn get_quotes_in_parallel<F>(
        &self,
        request: FiatQuoteOldRequest,
        fiat_mapping_map: HashMap<String, FiatMapping>,
        ip_address_info: IPAddressInfo,
        countries: Vec<FiatProviderCountry>,
        provider_priorities: &[PrimitiveFiatProvider],
        quote_fn: F,
        sort_fn: fn(&primitives::FiatQuoteOld, &primitives::FiatQuoteOld, &[PrimitiveFiatProvider]) -> std::cmp::Ordering,
    ) -> Result<FiatQuotesOld, Box<dyn Error + Send + Sync>>
    where
        F: Fn(
                &dyn FiatProvider,
                FiatQuoteOldRequest,
                FiatMapping,
            ) -> futures::future::BoxFuture<'_, Result<primitives::FiatQuoteOld, Box<dyn Error + Send + Sync>>>
            + Send
            + Sync,
    {
        let providers = self.get_providers_for_request(&request);
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
                            FiatQuoteError::UnsupportedCountryAsset(country_code, mapping.asset_symbol.symbol.clone()).to_string(),
                        ))
                    } else {
                        match Self::check_asset_limits_old(&request, &mapping) {
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

        quotes.sort_by(|a, b| sort_fn(a, b, provider_priorities));

        Ok(FiatQuotesOld { quotes, errors })
    }
}

#[allow(dead_code)]
fn precision(val: f64, precision: usize) -> f64 {
    format!("{val:.precision$}").parse::<f64>().unwrap()
}

fn sort_by_priority_then_amount<F>(
    a: &primitives::FiatQuoteOld,
    b: &primitives::FiatQuoteOld,
    get_amount: F,
    providers: &[PrimitiveFiatProvider],
) -> std::cmp::Ordering
where
    F: Fn(&primitives::FiatQuoteOld) -> f64,
{
    let a_amount = get_amount(a);
    let b_amount = get_amount(b);

    let a_priority = providers.iter().find(|p| p.id == a.provider.id).and_then(|p| p.priority).unwrap_or(0);
    let b_priority = providers.iter().find(|p| p.id == b.provider.id).and_then(|p| p.priority).unwrap_or(0);

    match (a_priority, b_priority) {
        (0, 0) => b_amount.partial_cmp(&a_amount).unwrap(),
        (0, _) => std::cmp::Ordering::Greater,
        (_, 0) => std::cmp::Ordering::Less,
        (a_pri, b_pri) if a_pri != b_pri => {
            let better_amount = b_amount.max(a_amount);
            let worse_amount = b_amount.min(a_amount);

            let higher_priority_id = if a_pri < b_pri { &a.provider.id } else { &b.provider.id };
            let threshold_bps = providers.iter().find(|p| &p.id == higher_priority_id).and_then(|p| p.threshold_bps);

            if let Some(threshold_bps) = threshold_bps {
                let threshold = threshold_bps as f64 / 10000.0;
                let diff_percent = (better_amount - worse_amount) / worse_amount;

                if diff_percent > threshold {
                    return b_amount.partial_cmp(&a_amount).unwrap();
                }
            }

            a_pri.cmp(&b_pri)
        }
        _ => b_amount.partial_cmp(&a_amount).unwrap(),
    }
}

fn sort_by_crypto_amount(a: &primitives::FiatQuoteOld, b: &primitives::FiatQuoteOld, providers: &[PrimitiveFiatProvider]) -> std::cmp::Ordering {
    sort_by_priority_then_amount(a, b, |q| q.crypto_amount, providers)
}

fn sort_by_fiat_amount(a: &primitives::FiatQuoteOld, b: &primitives::FiatQuoteOld, providers: &[PrimitiveFiatProvider]) -> std::cmp::Ordering {
    sort_by_priority_then_amount(a, b, |q| q.fiat_amount, providers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::FiatQuoteOld;

    #[test]
    fn test_precision() {
        assert_eq!(precision(1.123, 2), 1.12);
        assert_eq!(precision(1.123, 5), 1.123);
    }

    #[test]
    fn sort_quotes_by_priority() {
        let providers = vec![
            PrimitiveFiatProvider::mock_with_priority("moonpay", 1, None),
            PrimitiveFiatProvider::mock_with_priority("mercuryo", 2, None),
            PrimitiveFiatProvider::mock_with_priority("transak", 3, None),
        ];

        let moonpay = FiatQuoteOld::mock("moonpay", 0.45, 100.0);
        let mercuryo = FiatQuoteOld::mock("mercuryo", 0.48, 100.0);
        let transak = FiatQuoteOld::mock("transak", 0.47, 100.0);
        let paybis = FiatQuoteOld::mock("paybis", 0.50, 100.0);
        let banxa = FiatQuoteOld::mock("banxa", 0.40, 100.0);

        let mut quotes = [paybis.clone(), moonpay.clone(), banxa.clone(), transak.clone(), mercuryo.clone()];
        quotes.sort_by(|a, b| sort_by_crypto_amount(a, b, &providers));

        assert_eq!(quotes[0].provider.id, "moonpay");
        assert_eq!(quotes[1].provider.id, "mercuryo");
        assert_eq!(quotes[2].provider.id, "transak");
        assert_eq!(quotes[3].provider.id, "paybis");
        assert_eq!(quotes[4].provider.id, "banxa");
    }

    #[test]
    fn sort_quotes_with_threshold_override() {
        let providers = vec![
            PrimitiveFiatProvider::mock_with_priority("moonpay", 1, Some(1000)),
            PrimitiveFiatProvider::mock_with_priority("mercuryo", 2, Some(500)),
            PrimitiveFiatProvider::mock_with_priority("transak", 3, None),
        ];

        let moonpay = FiatQuoteOld::mock("moonpay", 0.45, 100.0);
        let mercuryo = FiatQuoteOld::mock("mercuryo", 0.48, 100.0);
        let transak = FiatQuoteOld::mock("transak", 0.60, 100.0);
        let paybis = FiatQuoteOld::mock("paybis", 0.52, 100.0);

        let mut quotes = [paybis.clone(), transak.clone(), mercuryo.clone(), moonpay.clone()];
        quotes.sort_by(|a, b| sort_by_crypto_amount(a, b, &providers));

        assert_eq!(quotes[0].provider.id, "transak");
        assert_eq!(quotes[1].provider.id, "moonpay");
        assert_eq!(quotes[2].provider.id, "mercuryo");
        assert_eq!(quotes[3].provider.id, "paybis");
    }
}

use cacher::{CacheKey, CacherClient};
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use crate::ip_check_client::{IPAddressInfo, IPCheckClient};
use crate::{
    CachedFiatQuoteData, FiatCacherClient, FiatProvider,
    model::{FiatMapping, FiatMappingMap},
};
use futures::future::join_all;
use primitives::{
    Asset, FiatAssetSymbol, FiatAssets, FiatProvider as PrimitiveFiatProvider, FiatProviderCountry, FiatQuote, FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlData,
    FiatQuotes,
};
use reqwest::Client as RequestClient;
use storage::{
    AssetFilter, AssetsRepository, Database, WalletsRepository,
    models::{FiatQuoteRequestRow, FiatQuoteRow, NewFiatWebhookRow},
};
use streamer::{FiatWebhook, FiatWebhookPayload, StreamProducer};

pub struct FiatClient {
    database: Database,
    cacher: CacherClient,
    fiat_cacher: FiatCacherClient,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
    ip_check_client: IPCheckClient,
    stream_producer: StreamProducer,
}

impl FiatClient {
    pub fn new(
        database: Database,
        cacher: CacherClient,
        providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
        ip_check_client: IPCheckClient,
        stream_producer: StreamProducer,
    ) -> Self {
        Self {
            database,
            cacher: cacher.clone(),
            fiat_cacher: FiatCacherClient::new(cacher),
            providers,
            ip_check_client,
            stream_producer,
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
        let assets = self.database.assets()?.get_assets_by_filter(vec![AssetFilter::IsBuyable(true)])?;
        Ok(FiatAssets {
            version: assets.clone().len() as u32,
            asset_ids: assets.into_iter().map(|x| x.asset.id.to_string()).collect::<Vec<String>>(),
        })
    }

    pub async fn get_off_ramp_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self.database.assets()?.get_assets_by_filter(vec![AssetFilter::IsSellable(true)])?;
        Ok(FiatAssets {
            version: assets.clone().len() as u32,
            asset_ids: assets.into_iter().map(|x| x.asset.id.to_string()).collect::<Vec<String>>(),
        })
    }

    pub async fn get_fiat_providers_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.fiat()?.get_fiat_providers_countries()?.into_iter().map(|r| r.as_primitive()).collect())
    }

    pub async fn get_order_status(&self, provider_name: &str, order_id: &str) -> Result<primitives::FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        self.provider(provider_name)?.get_order_status(order_id).await
    }

    pub async fn process_and_publish_webhook(&self, provider_name: &str, webhook_data: serde_json::Value) -> Result<FiatWebhookPayload, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider(provider_name)?;
        let webhook = provider.process_webhook(webhook_data.clone()).await;

        let (transaction_id, error) = match &webhook {
            Ok(FiatWebhook::OrderId(order_id)) => (Some(order_id.clone()), None),
            Ok(FiatWebhook::Transaction(tx)) => (Some(tx.provider_transaction_id.clone()), None),
            Ok(FiatWebhook::None) => (None, None),
            Err(e) => (None, Some(e.to_string())),
        };
        let webhook_row = NewFiatWebhookRow {
            provider: provider_name.to_string(),
            transaction_id,
            payload: webhook_data.clone(),
            error,
        };
        self.database.fiat()?.add_fiat_webhook(webhook_row)?;

        let webhook = match webhook {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to decode webhook payload: {}, JSON payload: {}", e, webhook_data);
                return Err(e);
            }
        };

        let payload = FiatWebhookPayload::new(provider.name(), webhook_data.clone(), webhook.clone());
        match webhook {
            FiatWebhook::OrderId(_) | FiatWebhook::Transaction(_) => {
                self.stream_producer.publish(streamer::QueueName::FiatOrderWebhooks, &payload).await?;
            }
            FiatWebhook::None => {}
        }
        Ok(payload)
    }

    fn get_fiat_mapping(&self, asset: &Asset, quote_type: &FiatQuoteType) -> Result<(FiatMappingMap, Vec<PrimitiveFiatProvider>), Box<dyn Error + Send + Sync>> {
        let fiat_assets = self.database.fiat()?.get_fiat_assets_for_asset_id(&asset.id.to_string())?;
        let providers = self
            .database
            .fiat()?
            .get_fiat_providers()?
            .into_iter()
            .map(|p| p.as_primitive())
            .collect::<Result<Vec<_>, _>>()?;

        let map: FiatMappingMap = fiat_assets
            .into_iter()
            .filter(|x| match quote_type {
                FiatQuoteType::Buy => x.is_buy_enabled(),
                FiatQuoteType::Sell => x.is_sell_enabled(),
            })
            .map(|x| {
                (
                    x.clone().provider,
                    FiatMapping {
                        asset: asset.clone(),
                        asset_symbol: FiatAssetSymbol {
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

    pub async fn get_quotes(&self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        let asset_id = request.asset_id.to_string();
        let asset = self.database.assets()?.get_asset(&asset_id)?;

        let fiat_providers_countries = self.get_fiat_providers_countries().await?;
        let ip_address_info = match self.get_ip_address(&request.ip_address).await {
            Ok(info) => info,
            Err(e) => {
                return Err(format!("IP address validation failed: {}", e).into());
            }
        };
        let (fiat_mapping_map, db_providers) = self.get_fiat_mapping(&asset, &request.quote_type)?;

        let provider_impls = self.get_providers(request.provider_id.clone());

        let futures = provider_impls.into_iter().filter_map(|provider| {
            let provider_id = provider.name().id().to_string();

            let db_provider = db_providers.iter().find(|p| p.id.as_ref() == provider_id)?;
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
                let asset = asset.clone();
                let mapping = mapping.clone();
                let country_code = ip_address_info.clone().alpha2;
                let provider_id_clone = provider_id.clone();
                let db_payment_methods = db_provider.payment_methods.clone();

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
                    let payment_methods = if !response.payment_methods.is_empty() {
                        response.payment_methods
                    } else if !db_payment_methods.is_empty() {
                        db_payment_methods
                    } else {
                        provider.payment_methods().await
                    };
                    let quote = FiatQuote::new(
                        response.quote_id,
                        asset,
                        provider.name().as_fiat_provider(),
                        quote_request.quote_type,
                        response.fiat_amount,
                        quote_request.currency,
                        response.crypto_amount,
                        latency,
                        payment_methods,
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

        let mut quotes: Vec<FiatQuote> = quotes.into_iter().map(|x| x.quote).collect();
        let sort_fn = match request.quote_type {
            FiatQuoteType::Buy => sort_quotes_by_crypto_amount_desc,
            FiatQuoteType::Sell => sort_quotes_by_crypto_amount_asc,
        };
        quotes.sort_by(|a, b| sort_fn(a, b, &db_providers));

        Ok(FiatQuotes { quotes, errors })
    }

    pub async fn get_quote_url(
        &self,
        quote_id: &str,
        wallet_id: i32,
        device_id: i32,
        ip_address: &str,
        locale: &str,
    ) -> Result<(FiatQuoteUrl, FiatQuote), Box<dyn Error + Send + Sync>> {
        let quote = self.fiat_cacher.get_quote(quote_id).await?;
        let provider = self.provider(quote.quote.provider.id.as_ref())?;

        let asset = &quote.quote.asset;

        let wallet_address = self.database.client()?.subscriptions_wallet_address_for_chain(device_id, wallet_id, asset.chain)?;

        let data = FiatQuoteUrlData {
            quote: quote.quote.clone(),
            asset_symbol: quote.asset_symbol,
            wallet_address,
            ip_address: ip_address.to_string(),
            locale: locale.to_string(),
        };

        let url = provider.get_quote_url(data).await?;

        let db_quote = FiatQuoteRow::from_primitive(&quote.quote);
        self.database.fiat()?.add_fiat_quotes(vec![db_quote])?;

        self.database.fiat()?.add_fiat_quote_request(FiatQuoteRequestRow {
            device_id,
            quote_id: quote_id.to_string(),
        })?;

        Ok((url, quote.quote))
    }

    pub async fn get_ip_address(&self, ip_address: &str) -> Result<IPAddressInfo, Box<dyn Error + Send + Sync>> {
        self.cacher
            .get_or_set_cached(CacheKey::FiatIpCheck(ip_address), || self.ip_check_client.get_ip_address(ip_address))
            .await
    }
}

use primitives::sort_by_priority_then_amount;

fn sort_quotes_by_crypto_amount_desc(a: &FiatQuote, b: &FiatQuote, providers: &[PrimitiveFiatProvider]) -> std::cmp::Ordering {
    sort_by_priority_then_amount(a.provider.id.as_ref(), b.provider.id.as_ref(), &a.crypto_amount, &b.crypto_amount, providers, false)
}

fn sort_quotes_by_crypto_amount_asc(a: &FiatQuote, b: &FiatQuote, providers: &[PrimitiveFiatProvider]) -> std::cmp::Ordering {
    sort_by_priority_then_amount(a.provider.id.as_ref(), b.provider.id.as_ref(), &a.crypto_amount, &b.crypto_amount, providers, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::FiatProviderName;

    fn mock_quote(provider: FiatProviderName, crypto_amount: f64) -> FiatQuote {
        let mut quote = FiatQuote::mock(provider);
        quote.crypto_amount = crypto_amount;
        quote
    }

    #[test]
    fn sort_buy_quotes_by_priority() {
        let providers = vec![
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, None),
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, None),
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::Transak, 3, None),
        ];

        let mut quotes = [
            mock_quote(FiatProviderName::Paybis, 0.50),
            mock_quote(FiatProviderName::MoonPay, 0.45),
            mock_quote(FiatProviderName::Banxa, 0.40),
            mock_quote(FiatProviderName::Transak, 0.47),
            mock_quote(FiatProviderName::Mercuryo, 0.48),
        ];
        quotes.sort_by(|a, b| sort_quotes_by_crypto_amount_desc(a, b, &providers));

        assert_eq!(quotes[0].provider.id, FiatProviderName::MoonPay);
        assert_eq!(quotes[1].provider.id, FiatProviderName::Mercuryo);
        assert_eq!(quotes[2].provider.id, FiatProviderName::Transak);
        assert_eq!(quotes[3].provider.id, FiatProviderName::Paybis);
        assert_eq!(quotes[4].provider.id, FiatProviderName::Banxa);
    }

    #[test]
    fn sort_buy_quotes_with_threshold_override() {
        let providers = vec![
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, Some(1000)),
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, Some(500)),
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::Transak, 3, None),
        ];

        let mut quotes = [
            mock_quote(FiatProviderName::Paybis, 0.52),
            mock_quote(FiatProviderName::Transak, 0.60),
            mock_quote(FiatProviderName::Mercuryo, 0.48),
            mock_quote(FiatProviderName::MoonPay, 0.45),
        ];
        quotes.sort_by(|a, b| sort_quotes_by_crypto_amount_desc(a, b, &providers));

        assert_eq!(quotes[0].provider.id, FiatProviderName::Transak);
        assert_eq!(quotes[1].provider.id, FiatProviderName::MoonPay);
        assert_eq!(quotes[2].provider.id, FiatProviderName::Mercuryo);
        assert_eq!(quotes[3].provider.id, FiatProviderName::Paybis);
    }

    #[test]
    fn sort_buy_quotes_by_amount_without_priority_override() {
        let providers = vec![PrimitiveFiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, Some(100))];

        let mut quotes = [
            mock_quote(FiatProviderName::MoonPay, 0.0773),
            mock_quote(FiatProviderName::Mercuryo, 0.0759),
            mock_quote(FiatProviderName::Transak, 0.07505),
            mock_quote(FiatProviderName::Paybis, 0.07721),
        ];

        quotes.sort_by(|a, b| sort_quotes_by_crypto_amount_desc(a, b, &providers));

        assert_eq!(quotes[0].provider.id, FiatProviderName::MoonPay);
        assert_eq!(quotes[1].provider.id, FiatProviderName::Paybis);
        assert_eq!(quotes[2].provider.id, FiatProviderName::Mercuryo);
        assert_eq!(quotes[3].provider.id, FiatProviderName::Transak);
    }

    #[test]
    fn sort_sell_quotes_by_crypto_amount_ascending() {
        let providers: Vec<PrimitiveFiatProvider> = vec![];

        let mut quotes = [
            mock_quote(FiatProviderName::MoonPay, 0.036108),
            mock_quote(FiatProviderName::Mercuryo, 0.03311059),
            mock_quote(FiatProviderName::Transak, 0.03086637),
        ];

        quotes.sort_by(|a, b| sort_quotes_by_crypto_amount_asc(a, b, &providers));

        assert_eq!(quotes[0].provider.id, FiatProviderName::Transak);
        assert_eq!(quotes[1].provider.id, FiatProviderName::Mercuryo);
        assert_eq!(quotes[2].provider.id, FiatProviderName::MoonPay);
    }
}

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
use gem_tracing::{error_with_fields, info_with_fields};
use number_formatter::BigNumberFormatter;
use primitives::{
    Asset, FiatAssetSymbol, FiatAssets, FiatProvider as PrimitiveFiatProvider, FiatProviderCountry, FiatQuote, FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlData,
    FiatQuotes, FiatTransaction, PaymentType,
};
use reqwest::Client as RequestClient;
use storage::{AssetFilter, AssetsRepository, Database, FiatRepository, WalletsRepository};
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
        Ok(FiatRepository::get_fiat_providers_countries(&mut self.database.fiat()?)?)
    }

    pub async fn get_order_status(&self, provider_name: &str, order_id: &str) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider(provider_name)?;
        let update = provider.get_order_status(order_id).await?;
        let transaction = self.database.fiat()?.update_fiat_transaction(provider.name(), update)?;
        Ok(transaction.as_primitive()?)
    }

    pub async fn process_and_publish_webhook(&self, provider_name: &str, webhook_data: serde_json::Value) -> Result<FiatWebhookPayload, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider(provider_name)?;
        let provider_id = provider.name().id().to_string();
        let webhook = match provider.process_webhook(webhook_data.clone()).await {
            Ok(webhook) => webhook,
            Err(e) => {
                error_with_fields!("failed to decode fiat webhook", &*e, provider = provider_id);
                return Err(e);
            }
        };

        let (kind, transaction_id) = match &webhook {
            FiatWebhook::OrderId(order_id) => ("order_id", Some(order_id.clone())),
            FiatWebhook::Transaction(tx) => ("transaction", tx.provider_transaction_id.clone().or(Some(tx.transaction_id.clone()))),
            FiatWebhook::None => ("none", None),
        };

        info_with_fields!(
            "received fiat webhook",
            provider = provider_id,
            kind = kind,
            transaction_id = transaction_id.as_deref().unwrap_or("")
        );

        let payload = FiatWebhookPayload::new(provider.name(), webhook_data.clone(), webhook.clone());
        match webhook {
            FiatWebhook::OrderId(_) | FiatWebhook::Transaction(_) => {
                self.stream_producer.publish(streamer::QueueName::FiatOrderWebhooks, &payload).await?;
                info_with_fields!("published fiat webhook", provider = provider_id, transaction_id = transaction_id.as_deref().unwrap_or(""));
            }
            FiatWebhook::None => {
                info_with_fields!("ignored fiat webhook", provider = provider_id);
            }
        }
        Ok(payload)
    }

    fn get_fiat_mapping(&self, asset: &Asset, quote_type: &FiatQuoteType) -> Result<(FiatMappingMap, Vec<PrimitiveFiatProvider>), Box<dyn Error + Send + Sync>> {
        let fiat_assets = self.database.fiat()?.get_fiat_assets_for_asset_id(&asset.id.to_string())?;
        let providers = self.database.fiat()?.get_fiat_providers()?.into_iter().map(|p| p.as_primitive()).collect();

        let map: FiatMappingMap = fiat_assets
            .into_iter()
            .filter(|x| match quote_type {
                FiatQuoteType::Buy => x.is_buy_enabled(),
                FiatQuoteType::Sell => x.is_sell_enabled(),
            })
            .map(|x| {
                let provider_id = x.provider.0.id().to_string();
                (
                    provider_id,
                    FiatMapping {
                        asset: asset.clone(),
                        asset_symbol: FiatAssetSymbol {
                            symbol: x.symbol.clone(),
                            network: x.network.clone(),
                        },
                        unsupported_countries: x.unsupported_countries(),
                        buy_limits: x.buy_limits(),
                        sell_limits: x.sell_limits(),
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

        let country_code = &ip_address_info.alpha2;

        let futures = provider_impls.into_iter().filter_map(|provider| {
            let provider_name = provider.name();
            let provider_id = provider_name.id().to_string();
            let db_provider = db_providers.iter().find(|p| p.id == provider_name)?;

            let countries: HashSet<String> = fiat_providers_countries.iter().filter(|x| x.provider == provider_name).map(|x| x.alpha2.clone()).collect();

            let mapping = fiat_mapping_map.get(&provider_id);
            if !is_provider_eligible(db_provider, &countries, mapping, country_code, &request.quote_type) {
                return None;
            }
            let mapping = mapping.unwrap().clone();
            let request = request.clone();
            let asset = asset.clone();
            let db_payment_methods = db_provider.payment_methods.clone();
            let country_code = country_code.clone();

            Some(async move {
                get_provider_quote(provider, &request, &asset, &mapping, &db_payment_methods, country_code)
                    .await
                    .map_err(|e| (provider_id, e))
            })
        });

        let results = join_all(futures).await;

        let mut quotes = Vec::new();
        let mut errors = Vec::new();

        for result in results {
            match result {
                Ok(cached_quote) => quotes.push(cached_quote),
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

    pub async fn get_quote(&self, quote_id: &str) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
        Ok(self.fiat_cacher.get_quote(quote_id).await?.quote)
    }

    pub async fn get_quote_url(
        &self,
        quote_id: &str,
        wallet_id: i32,
        device_id: i32,
        ip_address: &str,
        locale: &str,
    ) -> Result<(FiatQuoteUrl, FiatQuote), Box<dyn Error + Send + Sync>> {
        let crate::CachedFiatQuoteData {
            quote,
            asset_symbol,
            country_code,
        } = self.fiat_cacher.get_quote(quote_id).await?;
        let provider = self.provider(quote.provider.id.as_ref())?;
        let wallet_address_row = self.database.client()?.subscriptions_wallet_address_for_chain(device_id, wallet_id, quote.asset.chain)?;
        let data = FiatQuoteUrlData {
            quote: quote.clone(),
            asset_symbol,
            wallet_address: wallet_address_row.address,
            ip_address: ip_address.to_string(),
            locale: locale.to_string(),
        };

        let url = provider.get_quote_url(data.clone()).await?;
        let country = match country_code {
            Some(country_code) => Some(country_code),
            None => Some(self.get_ip_address(ip_address).await?.alpha2),
        };
        let pending_transaction = FiatTransaction::new_pending(&data, country, url.provider_transaction_id.clone());
        let pending_transaction_row = storage::models::NewFiatTransactionRow::new(pending_transaction, device_id, wallet_id, wallet_address_row.id);

        self.database.fiat()?.add_fiat_transaction(pending_transaction_row)?;

        Ok((url, quote))
    }

    pub async fn get_ip_address(&self, ip_address: &str) -> Result<IPAddressInfo, Box<dyn Error + Send + Sync>> {
        self.cacher
            .get_or_set_cached(CacheKey::FiatIpCheck(ip_address), || self.ip_check_client.get_ip_address(ip_address))
            .await
    }
}

fn is_provider_eligible(db_provider: &PrimitiveFiatProvider, countries: &HashSet<String>, mapping: Option<&FiatMapping>, country_code: &str, quote_type: &FiatQuoteType) -> bool {
    let is_enabled = match quote_type {
        FiatQuoteType::Buy => db_provider.is_buy_enabled(),
        FiatQuoteType::Sell => db_provider.is_sell_enabled(),
    };
    if !is_enabled {
        return false;
    }
    let Some(mapping) = mapping else {
        return false;
    };
    countries.contains(country_code) && !mapping.unsupported_countries.contains_key(country_code)
}

async fn get_provider_quote(
    provider: &(dyn FiatProvider + Send + Sync),
    request: &FiatQuoteRequest,
    asset: &Asset,
    mapping: &FiatMapping,
    db_payment_methods: &[PaymentType],
    country_code: String,
) -> Result<CachedFiatQuoteData, Box<dyn Error + Send + Sync>> {
    let start = std::time::Instant::now();
    let response = match request.quote_type {
        FiatQuoteType::Buy => provider.get_quote_buy(request.clone(), mapping.clone()).await,
        FiatQuoteType::Sell => provider.get_quote_sell(request.clone(), mapping.clone()).await,
    }?;

    if response.fiat_amount <= 0.0 || response.crypto_amount <= 0.0 {
        return Err("Invalid quote amounts".into());
    }

    let latency = start.elapsed().as_millis() as u64;
    let payment_methods = if !response.payment_methods.is_empty() {
        response.payment_methods
    } else if !db_payment_methods.is_empty() {
        db_payment_methods.to_vec()
    } else {
        provider.payment_methods().await
    };
    let value = quote_value(asset, response.crypto_amount)?;
    let quote = FiatQuote::new(
        response.quote_id,
        asset.clone(),
        provider.name().as_fiat_provider(),
        request.quote_type.clone(),
        response.fiat_amount,
        request.currency.clone(),
        response.crypto_amount,
        value,
        latency,
        payment_methods,
    );
    Ok(CachedFiatQuoteData {
        quote,
        asset_symbol: mapping.asset_symbol.clone(),
        country_code: Some(country_code),
    })
}

use primitives::sort_by_priority_then_amount;

fn sort_quotes_by_crypto_amount_desc(a: &FiatQuote, b: &FiatQuote, providers: &[PrimitiveFiatProvider]) -> std::cmp::Ordering {
    sort_by_priority_then_amount(a.provider.id.as_ref(), b.provider.id.as_ref(), &a.crypto_amount, &b.crypto_amount, providers, false)
}

fn sort_quotes_by_crypto_amount_asc(a: &FiatQuote, b: &FiatQuote, providers: &[PrimitiveFiatProvider]) -> std::cmp::Ordering {
    sort_by_priority_then_amount(a.provider.id.as_ref(), b.provider.id.as_ref(), &a.crypto_amount, &b.crypto_amount, providers, true)
}

fn quote_value(asset: &Asset, crypto_amount: f64) -> Result<String, Box<dyn Error + Send + Sync>> {
    let amount = format!("{crypto_amount:.precision$}", precision = asset.decimals as usize);
    Ok(BigNumberFormatter::value_from_amount(&amount, asset.decimals as u32)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::FiatProviderName;

    fn mock_quote(provider: FiatProviderName, crypto_amount: f64) -> FiatQuote {
        let mut quote = FiatQuote::mock(provider);
        quote.crypto_amount = crypto_amount;
        quote.value = quote_value(&quote.asset, crypto_amount).unwrap();
        quote
    }

    #[test]
    fn quote_value_uses_asset_precision_for_small_amounts() {
        let asset = Asset::from_chain(primitives::Chain::Ethereum);
        assert_eq!(quote_value(&asset, 0.000000000000000001_f64).unwrap(), "1");
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
            mock_quote(FiatProviderName::Flashnet, 0.40),
            mock_quote(FiatProviderName::Transak, 0.47),
            mock_quote(FiatProviderName::Mercuryo, 0.48),
        ];
        quotes.sort_by(|a, b| sort_quotes_by_crypto_amount_desc(a, b, &providers));

        assert_eq!(quotes[0].provider.id, FiatProviderName::MoonPay);
        assert_eq!(quotes[1].provider.id, FiatProviderName::Mercuryo);
        assert_eq!(quotes[2].provider.id, FiatProviderName::Transak);
        assert_eq!(quotes[3].provider.id, FiatProviderName::Paybis);
        assert_eq!(quotes[4].provider.id, FiatProviderName::Flashnet);
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

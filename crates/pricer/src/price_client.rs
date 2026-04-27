use crate::PriceProviders;
use cacher::{CacheError, CacheKey, CacherClient};
use gem_tracing::error_with_fields;
use prices::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider};
use primitives::{AssetId, AssetMarketPrice, AssetPriceInfo, AssetPrices, ChartTimeframe, FiatRate, PriceData, PriceId, PriceProvider};
use std::collections::HashSet;
use std::error::Error;
use storage::models::{NewPriceRow, PriceAssetRow};
use storage::{AssetsRepository, ChartsRepository, Database, PricesRepository};

#[derive(Clone)]
pub struct PriceClient {
    database: Database,
    cacher_client: CacherClient,
}

impl PriceClient {
    pub fn new(database: Database, cacher_client: CacherClient) -> Self {
        Self { database, cacher_client }
    }

    pub async fn set_fiat_rates(&self, rates: Vec<FiatRate>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let count = self
            .database
            .fiat()?
            .set_fiat_rates(rates.clone().into_iter().map(storage::models::FiatRateRow::from_primitive).collect())?;

        self.set_cache_fiat_rates(rates).await?;

        Ok(count)
    }

    pub fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.fiat()?.get_fiat_rates()?.into_iter().map(|r| r.as_primitive()).collect())
    }

    pub fn get_fiat_rate(&self, symbol: &str) -> Result<FiatRate, Box<dyn Error + Send + Sync>> {
        Ok(self.database.fiat()?.get_fiat_rate(symbol)?.as_primitive())
    }

    pub async fn get_asset_price(&self, asset_id: &AssetId, currency: &str) -> Result<AssetMarketPrice, Box<dyn Error + Send + Sync>> {
        let rate = self.get_fiat_rate(currency)?.rate;
        let price = self.get_cache_price(asset_id).await?;
        let prices = self
            .database
            .prices()?
            .get_prices_for_asset(asset_id)?
            .into_iter()
            .map(|row| row.as_primitive().with_rate(rate))
            .collect();
        Ok(AssetMarketPrice {
            price: Some(price.as_price_primitive_with_rate(rate)),
            market: Some(price.as_market_with_rate(rate)),
            prices: Some(prices),
        })
    }

    pub async fn set_cache_fiat_rates(&self, rates: Vec<FiatRate>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher_client.set_cached(CacheKey::FiatRates, &rates).await
    }

    pub async fn get_cache_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        match self.cacher_client.get_cached_optional::<Vec<FiatRate>>(CacheKey::FiatRates).await? {
            Some(rates) => Ok(rates),
            None => Err(Box::new(CacheError::not_found_resource("FiatRates"))),
        }
    }

    pub async fn set_cache_prices(&self, prices: Vec<AssetPriceInfo>, ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values: Vec<(String, String)> = prices
            .iter()
            .map(|x| (CacheKey::Price(&x.asset_id.to_string()).key(), serde_json::to_string(&x).unwrap()))
            .collect();

        self.cacher_client.set_values_with_publish(values, ttl_seconds).await
    }

    pub async fn get_cache_prices(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        let keys: Vec<String> = asset_ids.iter().map(|x| CacheKey::Price(&x.to_string()).key()).collect();
        self.cacher_client.get_values(keys).await
    }

    pub async fn get_cache_price(&self, asset_id: &AssetId) -> Result<AssetPriceInfo, Box<dyn Error + Send + Sync>> {
        let id = asset_id.to_string();
        match self.cacher_client.get_cached_optional::<AssetPriceInfo>(CacheKey::Price(&id)).await? {
            Some(price) => Ok(price),
            None => Err(Box::new(CacheError::not_found("Price", id))),
        }
    }

    pub async fn get_asset_prices(&self, currency: &str, asset_ids: Vec<AssetId>) -> Result<AssetPrices, Box<dyn Error + Send + Sync>> {
        let rate = self.get_fiat_rate(currency)?.rate;
        let prices = self
            .get_cache_prices(asset_ids)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|x| x.as_asset_price_primitive_with_rate(rate))
            .collect();

        Ok(AssetPrices {
            currency: currency.to_string(),
            prices,
        })
    }

    pub async fn aggregate_charts(&self, timeframe: ChartTimeframe) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.charts()?.aggregate_charts(timeframe)?)
    }

    pub async fn delete_charts(&self, timeframe: ChartTimeframe, before: chrono::NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.charts()?.delete_charts(timeframe, before)?)
    }

    pub async fn track_observed_assets(&self, asset_ids: &[AssetId]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let key = CacheKey::ObservedAssets;
        let ids: Vec<String> = asset_ids.iter().map(|id| id.to_string()).collect();
        self.cacher_client.sorted_set_incr_with_expire(&key.key(), &ids, key.ttl() as i64).await
    }

    pub async fn add_prices(&self, provider: &dyn PriceAssetsProvider, mappings: Vec<AssetPriceMapping>) -> Result<Vec<PriceData>, Box<dyn Error + Send + Sync>> {
        let mappings = self.filter_existing_assets(mappings)?;
        if mappings.is_empty() {
            return Ok(vec![]);
        }
        let prices = provider.get_prices(mappings).await?;
        if prices.is_empty() {
            return Ok(vec![]);
        }
        self.save_prices(provider.provider(), &prices)?;
        Ok(prices.iter().map(AssetPriceFull::as_price_data).collect())
    }

    pub async fn add_prices_for_asset_id(&self, providers: &PriceProviders, asset_id: &AssetId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let asset_id_str = asset_id.to_string();
        let mut count = 0;
        for provider in providers.values() {
            match self.add_prices_with_mappings(provider.as_ref(), provider.get_mappings_for_asset_id(asset_id).await).await {
                Ok(added) => count += added,
                Err(err) => {
                    let kind = provider.provider();
                    error_with_fields!("fetch prices provider failed", &*err, provider = kind.id(), asset_id = asset_id_str.as_str());
                }
            }
        }
        Ok(count)
    }

    pub async fn add_prices_for_price_id(&self, providers: &PriceProviders, price_id: &PriceId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let Some(provider) = providers.get(&price_id.provider) else {
            return Ok(0);
        };
        match self
            .add_prices_with_mappings(provider.as_ref(), provider.get_mappings_for_price_id(&price_id.provider_price_id).await)
            .await
        {
            Ok(added) => Ok(added),
            Err(err) => {
                let kind = provider.provider();
                let price_id_str = price_id.to_string();
                error_with_fields!("fetch prices provider failed", &*err, provider = kind.id(), price_id = price_id_str.as_str());
                Ok(0)
            }
        }
    }

    async fn add_prices_with_mappings(
        &self,
        provider: &dyn PriceAssetsProvider,
        mappings: Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.add_prices(provider, mappings?).await?.len())
    }

    pub fn filter_existing_assets(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }
        let asset_ids: Vec<AssetId> = mappings.iter().map(|m| m.asset_id.clone()).collect();
        let existing: HashSet<AssetId> = self.database.assets()?.get_assets_rows(asset_ids)?.into_iter().map(|a| a.as_asset_id()).collect();
        Ok(mappings.into_iter().filter(|m| existing.contains(&m.asset_id)).collect())
    }

    pub fn save_prices(&self, provider: PriceProvider, prices: &[AssetPriceFull]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let new_prices: Vec<NewPriceRow> = prices
            .iter()
            .map(|p| {
                NewPriceRow::with_market_data(
                    provider,
                    p.mapping.provider_price_id.clone(),
                    p.market.as_ref(),
                    Some(p.price.price),
                    Some(p.price.price_change_percentage_24h),
                )
            })
            .collect();
        let asset_rows: Vec<PriceAssetRow> = prices
            .iter()
            .map(|p| PriceAssetRow::new(p.mapping.asset_id.clone(), provider, &p.mapping.provider_price_id))
            .collect();
        self.database.prices()?.add_prices(new_prices)?;
        self.database.prices()?.set_prices_assets(asset_rows)?;
        Ok(prices.len())
    }
}

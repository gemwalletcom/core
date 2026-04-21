use gem_tracing::info_with_fields;
use prices::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use storage::database::prices::PriceFilter;
use storage::models::{NewPriceRow, PriceAssetRow, PriceRow};
use storage::{AssetsRepository, Database, PricesRepository};
use streamer::{PricesPayload, StreamProducer, StreamProducerQueue};

const BATCH_SIZE: usize = 1000;

pub struct PricesUpdater {
    provider: Arc<dyn PriceAssetsProvider>,
    database: Database,
    stream_producer: StreamProducer,
}

impl PricesUpdater {
    pub fn new(provider: Arc<dyn PriceAssetsProvider>, database: Database, stream_producer: StreamProducer) -> Self {
        Self {
            provider,
            database,
            stream_producer,
        }
    }

    pub async fn update_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        self.save_mappings(self.provider.get_assets().await?).await
    }

    pub async fn update_assets_new(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        self.save_mappings(self.provider.get_assets_new().await?).await
    }

    pub async fn update_prices_all(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider.provider();
        let prefix = format!("{}_", provider.id());
        let mappings: Vec<AssetPriceMapping> = self
            .database
            .prices()?
            .get_prices_assets_by_provider(provider)?
            .into_iter()
            .filter_map(|a| a.price_id.strip_prefix(&prefix).map(|id| AssetPriceMapping::new(a.asset_id.0, id.to_string())))
            .collect();
        self.update_prices(mappings).await
    }

    pub async fn update_prices_window(&self, offset: usize, limit: usize) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider.provider();
        let prefix = format!("{}_", provider.id());
        let synthetic_ids: Vec<String> = self
            .database
            .prices()?
            .get_prices_by_filter(vec![PriceFilter::Provider(provider)])?
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|p| p.id)
            .collect();
        if synthetic_ids.is_empty() {
            return Ok(0);
        }
        let mappings: Vec<AssetPriceMapping> = self
            .database
            .prices()?
            .get_prices_assets_for_price_ids(synthetic_ids)?
            .into_iter()
            .filter_map(|a| a.price_id.strip_prefix(&prefix).map(|id| AssetPriceMapping::new(a.asset_id.0, id.to_string())))
            .collect();
        self.update_prices(mappings).await
    }

    pub async fn update_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(0);
        }
        self.publish_prices(self.provider.get_prices(mappings).await?).await
    }

    async fn save_mappings(&self, mappings: Vec<AssetPriceMapping>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider.provider();
        let mut saved = 0;
        let mut queued = 0;

        for chunk in mappings.chunks(BATCH_SIZE) {
            let asset_ids: Vec<String> = chunk.iter().map(|m| m.asset_id.to_string()).collect();
            let existing: HashSet<String> = self.database.assets()?.get_assets_basic(asset_ids)?.into_iter().map(|a| a.asset.id.to_string()).collect();
            let (known, missing): (Vec<&AssetPriceMapping>, Vec<&AssetPriceMapping>) = chunk.iter().partition(|m| existing.contains(&m.asset_id.to_string()));

            if !missing.is_empty() {
                self.stream_producer.publish_fetch_assets(missing.iter().map(|m| m.asset_id.clone()).collect()).await?;
                queued += missing.len();
            }
            if known.is_empty() {
                continue;
            }

            let unique: HashMap<String, &AssetPriceMapping> = known.iter().map(|m| (m.asset_id.to_string(), *m)).collect();

            let new_prices: Vec<NewPriceRow> = unique
                .values()
                .map(|m| (m.provider_price_id.clone(), NewPriceRow::new(provider, m.provider_price_id.clone())))
                .collect::<HashMap<_, _>>()
                .into_values()
                .collect();
            self.database.prices()?.add_prices(new_prices)?;

            let asset_rows: Vec<PriceAssetRow> = unique.values().map(|m| PriceAssetRow::new(m.asset_id.clone(), provider, &m.provider_price_id)).collect();
            self.database.prices()?.set_prices_assets(asset_rows)?;
            saved += unique.len();
        }

        info_with_fields!("update prices assets", provider = provider.id(), saved = saved, queued_for_fetch = queued);
        Ok(saved)
    }

    async fn publish_prices(&self, prices: Vec<AssetPriceFull>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if prices.is_empty() {
            return Ok(0);
        }
        let provider = self.provider.provider();
        let rows: Vec<PriceRow> = prices
            .into_iter()
            .map(|p| price_row_from(provider, p))
            .map(|row| (row.id.clone(), row))
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();
        let count = rows.len();
        for chunk in rows.chunks(BATCH_SIZE) {
            let payload: Vec<_> = chunk.iter().map(|p| p.as_price_data()).collect();
            self.stream_producer.publish_prices(PricesPayload::new(payload)).await?;
        }

        info_with_fields!("update prices", provider = provider.id(), count = count);
        Ok(count)
    }
}

fn price_row_from(provider: PriceProvider, full: AssetPriceFull) -> PriceRow {
    let market = full.market.unwrap_or_default();

    PriceRow::new(
        provider,
        full.mapping.provider_price_id,
        full.price.price,
        full.price.price_change_percentage_24h,
        market.all_time_high.unwrap_or_default(),
        market.all_time_high_date.map(|d| d.naive_utc()),
        market.all_time_low.unwrap_or_default(),
        market.all_time_low_date.map(|d| d.naive_utc()),
        market.market_cap.unwrap_or_default(),
        market.market_cap_fdv.unwrap_or_default(),
        market.market_cap_rank.unwrap_or_default(),
        market.total_volume.unwrap_or_default(),
        market.circulating_supply.unwrap_or_default(),
        market.total_supply.unwrap_or_default(),
        market.max_supply.unwrap_or_default(),
        full.price.updated_at.naive_utc(),
    )
}

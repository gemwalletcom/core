use gem_tracing::info_with_fields;
use prices::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProviderAsset};
use primitives::{AssetId, PriceData};
use std::collections::HashMap;
use std::sync::Arc;
use storage::AssetUpdate;
use storage::database::prices::PriceFilter;
use storage::models::{AssetRow, NewPriceRow, PriceAssetRow};
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
        self.save_assets(self.provider.get_assets().await?).await
    }

    pub async fn update_assets_new(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        self.save_assets(self.provider.get_assets_new().await?).await
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

    async fn save_assets(&self, assets: Vec<PriceProviderAsset>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider.provider();
        let mut saved = 0;
        let mut queued = 0;

        for chunk in assets.chunks(BATCH_SIZE) {
            let asset_ids: Vec<String> = chunk.iter().map(|a| a.mapping.asset_id.to_string()).collect();
            let existing: HashMap<String, AssetRow> = self.database.assets()?.get_assets_rows(asset_ids)?.into_iter().map(|a| (a.id.clone(), a)).collect();
            let (known, missing): (Vec<&PriceProviderAsset>, Vec<&PriceProviderAsset>) = chunk.iter().partition(|a| existing.contains_key(&a.mapping.asset_id.to_string()));

            if !missing.is_empty() {
                self.stream_producer
                    .publish_fetch_assets(missing.iter().map(|a| a.mapping.asset_id.clone()).collect())
                    .await?;
                queued += missing.len();
            }
            if known.is_empty() {
                continue;
            }

            let unique: HashMap<String, &PriceProviderAsset> = known.iter().map(|a| (a.mapping.asset_id.to_string(), *a)).collect();

            let new_prices: Vec<NewPriceRow> = unique
                .values()
                .map(|a| (a.mapping.provider_price_id.clone(), NewPriceRow::new(provider, a.mapping.provider_price_id.clone())))
                .collect::<HashMap<_, _>>()
                .into_values()
                .collect();
            self.database.prices()?.add_prices(new_prices)?;

            let asset_rows: Vec<PriceAssetRow> = unique
                .values()
                .map(|a| PriceAssetRow::new(a.mapping.asset_id.clone(), provider, &a.mapping.provider_price_id))
                .collect();
            self.database.prices()?.set_prices_assets(asset_rows)?;

            let supply_updates: Vec<_> = unique
                .values()
                .filter_map(|asset| asset_supply_update(asset, existing.get(&asset.mapping.asset_id.to_string())?))
                .collect();
            self.store_asset_updates(supply_updates)?;
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

        let payload: Vec<PriceData> = prices
            .iter()
            .map(AssetPriceFull::as_price_data)
            .map(|data| (data.id.clone(), data))
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();
        let count = payload.len();
        for chunk in payload.chunks(BATCH_SIZE) {
            self.stream_producer.publish_prices(PricesPayload::new(chunk.to_vec())).await?;
        }

        info_with_fields!("update prices", provider = provider.id(), count = count);
        Ok(count)
    }

    fn store_asset_updates(&self, updates: Vec<(AssetId, AssetUpdate)>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (asset_id, update) in updates {
            self.database.assets()?.update_assets(vec![asset_id.to_string()], vec![update])?;
        }
        Ok(())
    }
}

fn asset_supply_update(asset: &PriceProviderAsset, current: &AssetRow) -> Option<(AssetId, AssetUpdate)> {
    let market = asset.market.as_ref()?;
    let circulating = market.circulating_supply.filter(|v| *v > 0.0).or(current.circulating_supply);
    let total = market.total_supply.filter(|v| *v > 0.0).or(current.total_supply);
    let max = market.max_supply.filter(|v| *v > 0.0).or(current.max_supply);
    if circulating == current.circulating_supply && total == current.total_supply && max == current.max_supply {
        return None;
    }
    Some((asset.mapping.asset_id.clone(), AssetUpdate::supply(circulating, total, max)?))
}

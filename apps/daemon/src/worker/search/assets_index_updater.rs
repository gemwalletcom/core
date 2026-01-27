use std::collections::HashMap;

use chrono::{NaiveDateTime, Utc};
use primitives::ConfigKey;
use search_index::{ASSETS_INDEX_NAME, AssetDocument, SearchIndexClient, sanitize_index_primary_id};
use storage::models::PriceAssetDataRow;
use storage::{AssetsUsageRanksRepository, ConfigCacher, Database, PricesRepository, TagRepository};

pub struct AssetsIndexUpdater {
    database: Database,
    config: ConfigCacher,
    search_index: SearchIndexClient,
}

impl AssetsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        let config = ConfigCacher::new(database.clone());
        Self {
            database,
            config,
            search_index: search_index.clone(),
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self.database.prices()?.get_prices_assets_list()?;
        let assets_tags = self.database.tag()?.get_assets_tags()?;
        let usage_ranks = self.database.assets_usage_ranks()?.get_all_usage_ranks()?;

        let assets_tags_map: HashMap<String, Vec<String>> = assets_tags.into_iter().fold(HashMap::new(), |mut acc, tag| {
            acc.entry(tag.asset_id).or_default().push(tag.tag_id);
            acc
        });
        let usage_ranks_map: HashMap<String, i32> = usage_ranks.into_iter().map(|r| (r.asset_id, r.usage_rank)).collect();

        let now = Utc::now().naive_utc();
        let last_updated_at = self.config.get_datetime(ConfigKey::SearchAssetsLastUpdatedAt)?;

        let updated_prices: Vec<_> = prices.into_iter().filter(|x| Self::is_updated(x, last_updated_at)).collect();
        let documents = Self::build_documents(&updated_prices, &assets_tags_map, &usage_ranks_map);
        let count = documents.len();
        if count > 0 {
            self.search_index.add_documents(ASSETS_INDEX_NAME, documents).await?;
        }

        self.config.set_datetime(ConfigKey::SearchAssetsLastUpdatedAt, now)?;
        Ok(count)
    }

    fn is_updated(data: &PriceAssetDataRow, since: NaiveDateTime) -> bool {
        data.asset.updated_at > since || data.price.as_ref().is_some_and(|p| p.last_updated_at > since)
    }

    fn build_documents(prices: &[PriceAssetDataRow], assets_tags_map: &HashMap<String, Vec<String>>, usage_ranks_map: &HashMap<String, i32>) -> Vec<AssetDocument> {
        prices
            .iter()
            .map(|x| {
                let asset_id = x.asset.id.as_str();
                let usage_rank = usage_ranks_map.get(asset_id).copied().unwrap_or(0);
                AssetDocument {
                    id: sanitize_index_primary_id(asset_id),
                    asset: x.asset.as_primitive(),
                    properties: x.asset.clone().as_property_primitive(),
                    score: x.asset.clone().as_score_primitive(),
                    usage_rank,
                    market: x.price.as_ref().map(|price| price.as_market_primitive()),
                    tags: assets_tags_map.get(asset_id).cloned(),
                }
            })
            .collect()
    }
}

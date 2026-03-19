use std::collections::HashMap;

use super::sync::{SearchSyncClient, SearchSyncResult};
use primitives::ConfigKey;
use search_index::{ASSETS_INDEX_NAME, AssetDocument, SearchIndexClient, sanitize_index_primary_id};
use storage::models::PriceAssetDataRow;
use storage::{AssetsUsageRanksRepository, AssetsWithPricesFilter, Database, PricesRepository, TagRepository};

pub struct AssetsIndexUpdater {
    database: Database,
    sync_client: SearchSyncClient,
}

impl AssetsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            sync_client: SearchSyncClient::new(database.clone(), search_index),
            database,
        }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let sync = self.sync_client.for_key(ConfigKey::SearchAssetsLastUpdatedAt)?;
        let filters = sync.since().map(AssetsWithPricesFilter::UpdatedSince).into_iter().collect();
        let prices = self.database.prices()?.get_assets_with_prices_by_filter(filters)?;

        if prices.is_empty() {
            return sync.write(ASSETS_INDEX_NAME, Vec::<AssetDocument>::new()).await;
        }

        let assets_tags = self.database.tag()?.get_assets_tags()?;
        let usage_ranks = self.database.assets_usage_ranks()?.get_all_usage_ranks()?;

        let assets_tags_map: HashMap<String, Vec<String>> = assets_tags.into_iter().fold(HashMap::new(), |mut acc, tag| {
            acc.entry(tag.asset_id).or_default().push(tag.tag_id);
            acc
        });
        let usage_ranks_map: HashMap<String, i32> = usage_ranks.into_iter().map(|r| (r.asset_id, r.usage_rank)).collect();

        let documents = Self::build_documents(prices.iter(), &assets_tags_map, &usage_ranks_map);

        sync.write(ASSETS_INDEX_NAME, documents).await
    }

    fn build_documents<'a>(
        prices: impl IntoIterator<Item = &'a PriceAssetDataRow>,
        assets_tags_map: &HashMap<String, Vec<String>>,
        usage_ranks_map: &HashMap<String, i32>,
    ) -> Vec<AssetDocument> {
        prices
            .into_iter()
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

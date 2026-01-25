use std::collections::HashMap;

use search_index::{ASSETS_INDEX_NAME, AssetDocument, SearchIndexClient, sanitize_index_primary_id};
use storage::{AssetsUsageRanksRepository, Database, PricesRepository, TagRepository};

pub struct AssetsIndexUpdater {
    database: Database,
    search_index: SearchIndexClient,
}

impl AssetsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            database,
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

        let documents = prices
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
            .collect::<Vec<_>>();

        self.search_index.sync_documents(ASSETS_INDEX_NAME, documents, |doc| doc.id.clone()).await
    }
}

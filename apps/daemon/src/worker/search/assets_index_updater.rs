use std::collections::HashMap;

use search_index::{ASSETS_INDEX_NAME, AssetDocument, SearchIndexClient, sanitize_index_primary_id};
use storage::Database;

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

    pub async fn update(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self.database.client()?.prices().get_prices_assets_list()?;
        let assets_tags = self.database.client()?.tag().get_assets_tags()?;
        let assets_tags_map: HashMap<String, Vec<String>> = assets_tags.into_iter().fold(HashMap::new(), |mut acc, tag| {
            acc.entry(tag.asset_id).or_default().push(tag.tag_id);
            acc
        });

        let documents = prices
            .into_iter()
            .map(|x| AssetDocument {
                id: sanitize_index_primary_id(x.asset.id.as_str()),
                asset: x.asset.as_primitive(),
                properties: x.asset.clone().as_property_primitive(),
                score: x.asset.clone().as_score_primitive(),
                market: x.price.as_ref().map(|price| price.as_market_primitive()),
                tags: assets_tags_map.get(x.asset.id.as_str()).cloned(),
            })
            .collect::<Vec<_>>();

        self.search_index.sync_documents(ASSETS_INDEX_NAME, documents, |doc| doc.id.clone()).await
    }
}

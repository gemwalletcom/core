use std::collections::{HashMap, HashSet};

use search_index::{ASSETS_INDEX_NAME, AssetDocument, DocumentId, SearchIndexClient, sanitize_index_primary_id};
use storage::DatabaseClient;

pub struct AssetsIndexUpdater {
    database: DatabaseClient,
    search_index: SearchIndexClient,
}

impl AssetsIndexUpdater {
    pub fn new(database_url: &str, search_index: &SearchIndexClient) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
            search_index: search_index.clone(),
        }
    }

    pub async fn update(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self.database.prices().get_prices_assets_list()?;
        let assets_tags = self.database.tag().get_assets_tags()?;
        let assets_tags_map: HashMap<String, Vec<String>> = assets_tags.into_iter().fold(HashMap::new(), |mut acc, tag| {
            acc.entry(tag.asset_id).or_default().push(tag.tag_id);
            acc
        });

        let documents = prices
            .clone()
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
        self.search_index.add_documents(ASSETS_INDEX_NAME, documents.clone()).await?;

        // delete outdated documents
        let db_documents_ids: HashSet<_> = documents.iter().map(|x| x.id.clone()).collect();
        let existing_documents_ids: HashSet<_> = self
            .search_index
            .get_documents_all::<DocumentId>(ASSETS_INDEX_NAME)
            .await?
            .into_iter()
            .map(|x| x.id.clone())
            .collect();
        let stale_ids: Vec<&str> = existing_documents_ids.difference(&db_documents_ids).map(|id| id.as_str()).collect();

        self.search_index.delete_documents(ASSETS_INDEX_NAME, stale_ids).await?;

        Ok(documents.len())
    }
}

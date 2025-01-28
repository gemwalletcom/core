use search_index::{sanitize_index_primary_id, AssetDocument, SearchIndexClient, ASSETS_INDEX_NAME};
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
        let prices = self.database.get_prices_assets_list()?;

        let documents = prices
            .clone()
            .into_iter()
            .map(|x| AssetDocument {
                id: sanitize_index_primary_id(x.asset.id.as_str()),
                asset: x.asset.as_primitive(),
                properties: x.asset.clone().as_property_primitive(),
                score: x.asset.clone().as_score_primitive(),
                market: Some(x.price.as_market_primitive()),
            })
            .collect::<Vec<_>>();

        self.search_index.add_documents(ASSETS_INDEX_NAME, documents.clone()).await?;

        Ok(documents.len())
    }
}

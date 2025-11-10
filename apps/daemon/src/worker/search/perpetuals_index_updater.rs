use search_index::{PERPETUALS_INDEX_NAME, PerpetualDocument, SearchIndexClient};
use storage::Database;

pub struct PerpetualsIndexUpdater {
    database: Database,
    search_index: SearchIndexClient,
}

impl PerpetualsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            database,
            search_index: search_index.clone(),
        }
    }

    pub async fn update(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let perpetuals = self.database.client()?.perpetuals().perpetuals_all()?;
        let asset_ids = perpetuals.iter().map(|p| p.asset_id.to_string()).collect::<Vec<_>>();
        let assets = self.database.client()?.assets().get_assets(asset_ids)?;

        let assets_map = assets.into_iter().map(|a| (a.id.to_string(), a)).collect::<std::collections::HashMap<_, _>>();

        let documents: Vec<PerpetualDocument> = perpetuals
            .into_iter()
            .filter_map(|perpetual| {
                assets_map
                    .get(&perpetual.asset_id.to_string())
                    .map(|asset| (perpetual.clone(), asset.clone()).into())
            })
            .collect();

        self.search_index.sync_documents(PERPETUALS_INDEX_NAME, documents, |doc| doc.id.clone()).await
    }
}

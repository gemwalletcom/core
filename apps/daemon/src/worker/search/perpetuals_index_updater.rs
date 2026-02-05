use std::collections::HashMap;

use chrono::Utc;
use primitives::ConfigKey;
use search_index::{PERPETUALS_INDEX_NAME, PerpetualDocument, SearchIndexClient};
use storage::models::{AssetRow, PerpetualRow};
use storage::{AssetsRepository, ConfigCacher, Database, PerpetualsRepository};

pub struct PerpetualsIndexUpdater {
    database: Database,
    config: ConfigCacher,
    search_index: SearchIndexClient,
}

impl PerpetualsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        let config = ConfigCacher::new(database.clone());
        Self {
            database,
            config,
            search_index: search_index.clone(),
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let perpetuals = self.database.perpetuals()?.perpetuals_all_rows()?;
        let asset_ids = perpetuals.iter().map(|p| p.asset_id.to_string()).collect::<Vec<_>>();
        let assets = self.database.assets()?.get_assets_rows(asset_ids)?;

        let assets_map: HashMap<String, AssetRow> = assets.into_iter().map(|a| (a.id.to_string(), a)).collect();

        let now = Utc::now().naive_utc();
        let last_updated_at = self.config.get_datetime(ConfigKey::SearchPerpetualsLastUpdatedAt)?;

        let updated_perpetuals: Vec<_> = perpetuals.into_iter().filter(|p| p.updated_at > last_updated_at).collect();
        let documents = Self::build_documents(&updated_perpetuals, &assets_map);
        let count = documents.len();
        if count > 0 {
            self.search_index.add_documents(PERPETUALS_INDEX_NAME, documents).await?;
        }

        self.config.set_datetime(ConfigKey::SearchPerpetualsLastUpdatedAt, now)?;
        Ok(count)
    }

    fn build_documents(perpetuals: &[PerpetualRow], assets_map: &HashMap<String, AssetRow>) -> Vec<PerpetualDocument> {
        perpetuals
            .iter()
            .filter_map(|p| assets_map.get(&p.asset_id).map(|a| (p.as_primitive(), a.as_primitive()).into()))
            .collect()
    }
}

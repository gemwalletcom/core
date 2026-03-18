use std::collections::HashMap;

use super::sync::{SearchSyncClient, SearchSyncResult};
use primitives::ConfigKey;
use search_index::{PERPETUALS_INDEX_NAME, PerpetualDocument, SearchIndexClient};
use storage::models::{AssetRow, PerpetualRow};
use storage::{AssetsRepository, Database, PerpetualsRepository};

pub struct PerpetualsIndexUpdater {
    database: Database,
    sync_client: SearchSyncClient,
}

impl PerpetualsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            sync_client: SearchSyncClient::new(database.clone(), search_index),
            database,
        }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let perpetuals = self.database.perpetuals()?.perpetuals_all_rows()?;
        let asset_ids = perpetuals.iter().map(|p| p.asset_id.to_string()).collect::<Vec<_>>();
        let assets = self.database.assets()?.get_assets_rows(asset_ids)?;

        let assets_map: HashMap<String, AssetRow> = assets.into_iter().map(|a| (a.id.to_string(), a)).collect();

        let sync = self.sync_client.for_key(ConfigKey::SearchPerpetualsLastUpdatedAt)?;
        let documents = if sync.should_replace_index() {
            Self::build_documents(perpetuals.iter(), &assets_map)
        } else {
            Self::build_documents(perpetuals.iter().filter(|p| p.updated_at > sync.last_updated_at()), &assets_map)
        };

        sync.write(PERPETUALS_INDEX_NAME, documents).await
    }

    fn build_documents<'a>(perpetuals: impl IntoIterator<Item = &'a PerpetualRow>, assets_map: &HashMap<String, AssetRow>) -> Vec<PerpetualDocument> {
        perpetuals
            .into_iter()
            .filter_map(|p| assets_map.get(&p.asset_id).map(|a| (p.as_primitive(), a.as_primitive()).into()))
            .collect()
    }
}

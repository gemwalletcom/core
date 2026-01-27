use api_connector::StaticAssetsClient;
use futures::{StreamExt, stream};
use primitives::Chain;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use storage::{AssetFilter, AssetUpdate, AssetsRepository, Database};
use strum::IntoEnumIterator;

pub struct AssetsImagesUpdater {
    client: StaticAssetsClient,
    database: Database,
}

impl AssetsImagesUpdater {
    pub fn new(client: StaticAssetsClient, database: Database) -> Self {
        Self { client, database }
    }

    pub async fn update_assets_images(&self) -> Result<HashMap<&'static str, usize>, Box<dyn Error + Send + Sync>> {
        let new: HashSet<String> = stream::iter(Chain::iter())
            .map(|chain| async move {
                self.client
                    .get_assets_list(chain)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(move |addr| format!("{}_{}", chain.as_ref(), addr))
                    .collect::<Vec<_>>()
            })
            .buffer_unordered(6)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect();

        let current: HashSet<String> = self
            .database
            .assets()?
            .get_assets_by_filter(vec![AssetFilter::HasImage(true)])?
            .into_iter()
            .map(|a| a.asset.id.to_string())
            .collect();

        let additions: Vec<String> = new.difference(&current).cloned().collect();
        let removals: Vec<String> = current.difference(&new).cloned().collect();

        let result = HashMap::from([("additions", additions.len()), ("removals", removals.len())]);

        if !additions.is_empty() {
            self.database.assets()?.update_assets(additions, vec![AssetUpdate::HasImage(true)])?;
        }
        if !removals.is_empty() {
            self.database.assets()?.update_assets(removals, vec![AssetUpdate::HasImage(false)])?;
        }

        Ok(result)
    }
}

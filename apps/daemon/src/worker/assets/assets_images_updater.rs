use api_connector::StaticAssetsClient;
use primitives::Chain;
use std::collections::HashSet;
use std::error::Error;
use storage::{AssetFilter, AssetUpdate, AssetsRepository, Database};

pub struct AssetsImagesUpdater {
    client: StaticAssetsClient,
    database: Database,
}

impl AssetsImagesUpdater {
    pub fn new(client: StaticAssetsClient, database: Database) -> Self {
        Self { client, database }
    }

    pub async fn update_chain(&self, chain: Chain) -> Result<(usize, usize), Box<dyn Error + Send + Sync>> {
        let mut assets = self.client.get_assets_list(chain).await?;
        assets.push(chain.as_asset_id());
        let new: HashSet<String> = assets.into_iter().map(|id| id.to_string()).collect();

        let current: HashSet<String> = self
            .database
            .assets()?
            .get_assets_by_filter(vec![AssetFilter::HasImage(true), AssetFilter::Chain(chain.as_ref().to_string())])?
            .into_iter()
            .map(|a| a.asset.id.to_string())
            .collect();

        let additions: Vec<String> = new.difference(&current).cloned().collect();
        let removals: Vec<String> = current.difference(&new).cloned().collect();

        if !additions.is_empty() {
            self.database.assets()?.update_assets(additions.clone(), vec![AssetUpdate::HasImage(true)])?;
        }
        if !removals.is_empty() {
            self.database.assets()?.update_assets(removals.clone(), vec![AssetUpdate::HasImage(false)])?;
        }

        Ok((additions.len(), removals.len()))
    }
}

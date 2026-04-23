use std::collections::HashSet;
use std::error::Error;
use storage::{AssetFilter, AssetUpdate, AssetsRepository, Database, PricesRepository};

pub struct AssetsHasPriceUpdater {
    database: Database,
}

impl AssetsHasPriceUpdater {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn update(&self) -> Result<(usize, usize), Box<dyn Error + Send + Sync>> {
        let eligible: HashSet<String> = self.database.prices()?.get_prices_assets()?.into_iter().map(|a| a.asset_id.to_string()).collect();

        let current: HashSet<String> = self
            .database
            .assets()?
            .get_assets_by_filter(vec![AssetFilter::HasPrice(true)])?
            .into_iter()
            .map(|a| a.asset.id.to_string())
            .collect();

        let additions: Vec<String> = eligible.difference(&current).cloned().collect();
        let removals: Vec<String> = current.difference(&eligible).cloned().collect();

        if !additions.is_empty() {
            self.database.assets()?.update_assets(additions.clone(), vec![AssetUpdate::HasPrice(true)])?;
        }
        if !removals.is_empty() {
            self.database.assets()?.update_assets(removals.clone(), vec![AssetUpdate::HasPrice(false)])?;
        }

        Ok((additions.len(), removals.len()))
    }
}

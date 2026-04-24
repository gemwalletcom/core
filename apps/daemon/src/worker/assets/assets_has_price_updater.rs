use primitives::AssetId;
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
        let eligible: HashSet<AssetId> = self.database.prices()?.get_prices_assets()?.into_iter().map(|a| a.asset_id.0).collect();

        let current: HashSet<AssetId> = self
            .database
            .assets()?
            .get_assets_by_filter(vec![AssetFilter::HasPrice(true)])?
            .into_iter()
            .map(|a| a.asset.id)
            .collect();

        let additions: Vec<AssetId> = eligible.difference(&current).cloned().collect();
        let removals: Vec<AssetId> = current.difference(&eligible).cloned().collect();

        let additions_len = additions.len();
        let removals_len = removals.len();

        if !additions.is_empty() {
            self.database.assets()?.update_assets(additions, vec![AssetUpdate::HasPrice(true)])?;
        }
        if !removals.is_empty() {
            self.database.assets()?.update_assets(removals, vec![AssetUpdate::HasPrice(false)])?;
        }

        Ok((additions_len, removals_len))
    }
}

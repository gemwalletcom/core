use std::error::Error;

use primitives::FiatAssets;
use storage::Database;

#[derive(Clone)]
pub struct SwapClient {
    database: Database,
}

impl SwapClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_swap_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self.database.client()?.assets().get_swap_assets()?;
        let version = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() / 3600;

        Ok(FiatAssets {
            version: version as u32,
            asset_ids: assets,
        })
    }
}

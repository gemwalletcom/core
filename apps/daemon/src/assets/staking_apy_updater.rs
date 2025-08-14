use std::error::Error;

use primitives::Chain;
use settings_chain::ChainProviders;
use storage::{AssetUpdate, AssetsRepository, DatabaseClient};

pub struct StakeApyUpdater {
    chain_providers: ChainProviders,
    database: Box<DatabaseClient>,
}

impl StakeApyUpdater {
    pub fn new(chain_providers: ChainProviders, database_url: &str) -> Self {
        let database = Box::new(DatabaseClient::new(database_url));
        Self { chain_providers, database }
    }

    pub async fn update_staking_apy(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        for chain in Chain::stakeable() {
            match self.chain_providers.get_staking_apy(chain).await {
                Ok(apy) => {
                    let apy = (apy * 100.0).round() / 100.0;
                    self.database
                        .update_asset(chain.as_asset_id().to_string(), AssetUpdate::StakingApr(Some(apy)))?;
                    println!("update_staking_apy chain: {chain} apy: {apy}");
                }
                Err(e) => eprintln!("update_staking_apy chain: {chain} error {e}"),
            }
        }
        Ok(())
    }
}

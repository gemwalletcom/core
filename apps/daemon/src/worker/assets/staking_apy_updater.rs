use std::error::Error;

use gem_tracing::info_with_fields;
use primitives::Chain;
use settings_chain::ChainProviders;
use storage::Database;
use storage::AssetUpdate;

pub struct StakeApyUpdater {
    chain_providers: ChainProviders,
    database: Database,
}

impl StakeApyUpdater {
    pub fn new(chain_providers: ChainProviders, database: Database) -> Self {
        
        Self { chain_providers, database }
    }

    pub async fn update_staking_apy(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        for chain in Chain::stakeable() {
            match self.chain_providers.get_staking_apy(chain).await {
                Ok(apy) => {
                    let apy = (apy * 100.0).round() / 100.0;
                    self.database
                        .client()?
                        .assets()
                        .update_assets(vec![chain.as_asset_id().to_string()], vec![AssetUpdate::StakingApr(Some(apy))])?;
                    info_with_fields!("update_staking_apy chain", chain = chain.as_ref(), apy = apy);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

use std::error::Error;

use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{Chain, asset_score::AssetRank};
use settings::{Settings, service_user_agent};
use settings_chain::ProviderFactory;
use storage::Database;
use storage::{AssetUpdate, models::StoragePerpetual};

pub struct PerpetualUpdater {
    settings: Settings,
    database: Database,
}

impl PerpetualUpdater {
    pub fn new(settings: Settings, database: Database) -> Self {
        Self {
            settings,
            database,
        }
    }

    pub async fn update_perpetuals(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chains = [Chain::HyperCore];
        for chain in chains {
            let provider = ProviderFactory::new_from_settings_with_user_agent(chain, &self.settings, &service_user_agent("daemon", Some("perpetual_updater")));
            let perpetuals_data = provider.get_perpetuals_data().await?;

            let assets = perpetuals_data.iter().map(|x| x.asset.clone()).collect::<Vec<_>>();
            let asset_ids = assets.iter().map(|x| x.id.to_string()).collect::<Vec<_>>();
            let perpetuals = perpetuals_data
                .into_iter()
                .map(|x| StoragePerpetual::from_primitive(x.perpetual))
                .collect::<Vec<_>>();

            self.database.client()?.assets().upsert_assets(assets)?;
            self.database.client()?.assets().update_assets(
                asset_ids,
                vec![
                    AssetUpdate::Rank(AssetRank::Unknown.threshold()),
                    AssetUpdate::IsEnabled(false),
                    AssetUpdate::IsSwappable(false),
                    AssetUpdate::IsBuyable(false),
                    AssetUpdate::IsSellable(false),
                ],
            )?;

            match self.database.client()?.perpetuals().perpetuals_update(perpetuals.clone()) {
                Ok(_) => {
                    info_with_fields!("Updated perpetuals for chain", chain = &chain.to_string(), values = perpetuals.len());
                }
                Err(e) => {
                    error_with_fields!("Failed to update perpetuals for chain", &e, chain = chain.as_ref());
                }
            }
        }
        Ok(())
    }
}

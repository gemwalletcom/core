use std::error::Error;

use gem_tracing::{error_with_fields, info_with_fields};
use primitives::Chain;
use settings::{Settings, service_user_agent};
use settings_chain::ProviderFactory;
use storage::{DatabaseClient, models::StoragePerpetual};

pub struct PerpetualUpdater {
    settings: Settings,
    database: DatabaseClient,
}

impl PerpetualUpdater {
    pub fn new(settings: &Settings) -> Self {
        Self {
            settings: settings.clone(),
            database: DatabaseClient::new(&settings.postgres.url),
        }
    }

    pub async fn update_perpetuals(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chains = [Chain::HyperCore];
        for chain in chains {
            let provider = ProviderFactory::new_from_settings_with_user_agent(chain, &self.settings, &service_user_agent("daemon", Some("perpetual_updater")));
            let perpetuals = provider.get_perpetuals_data().await?;

            let values = perpetuals
                .into_iter()
                .map(|x| StoragePerpetual::from_primitive(x.perpetual))
                .collect::<Vec<_>>();

            match self.database.perpetuals().perpetuals_update(values.clone()) {
                Ok(_) => {
                    info_with_fields!("Updated perpetuals for chain", chain = &chain.to_string(), values = values.len());
                }
                Err(e) => {
                    error_with_fields!("Failed to update perpetuals for chain", &e, chain = chain.as_ref());
                }
            }
        }
        Ok(())
    }
}

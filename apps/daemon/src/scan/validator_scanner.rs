use api_connector::StaticAssetsClient;
use primitives::{Chain, StakeValidator};
use settings_chain::ChainProviders;
use std::error::Error;
use storage::{DatabaseClient, ScanAddressesRepository};

pub struct ValidatorScanner {
    chain_providers: ChainProviders,
    database: DatabaseClient,
    assets_url: String,
}

impl ValidatorScanner {
    pub fn new(chain_providers: ChainProviders, database_url: &str, assets_url: &str) -> Self {
        Self {
            chain_providers,
            database: DatabaseClient::new(database_url),
            assets_url: assets_url.to_string(),
        }
    }

    pub async fn update_validators(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chains = Chain::stakeable();

        for chain in chains {
            match self.chain_providers.get_validators(chain).await {
                Ok(validators) => {
                    let scan_addresses = validators.into_iter().filter_map(|v| v.as_scan_address(chain)).collect();
                    let count = self.database.add_scan_addresses(scan_addresses)?;
                    println!("update_validators: {chain}: {count}");
                }
                Err(e) => {
                    println!("update_validators: {chain}: error {e}");
                }
            }
        }

        Ok(())
    }

    pub async fn update_validators_from_static_assets(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chains = vec![Chain::Tron, Chain::SmartChain];
        let static_assets_client = StaticAssetsClient::new(&self.assets_url);

        for chain in chains {
            match static_assets_client.get_validators(chain).await {
                Ok(static_validators) => {
                    let validators: Vec<StakeValidator> = static_validators.into_iter().map(|v| StakeValidator::new(v.id, v.name)).collect();

                    let scan_addresses = validators.into_iter().filter_map(|v| v.as_scan_address(chain)).collect();
                    let count = self.database.add_scan_addresses(scan_addresses)?;
                    println!("update_validators: from_static_assets: {chain}: {count}");
                }
                Err(e) => {
                    println!("update_validators: from_static_assets: {chain}: error {e}");
                }
            }
        }

        Ok(())
    }
}

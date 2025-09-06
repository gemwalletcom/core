use api_connector::StaticAssetsClient;
use gem_tracing::{error_with_context, info_with_context};
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

    pub async fn update_validators(&mut self, name: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chains = Chain::stakeable();

        for chain in chains {
            match self.update_validators_for_chain(chain).await {
                Ok(count) => info_with_context(name, &[("chain", chain.as_ref()), ("count", &count.to_string())]),
                Err(e) => error_with_context(name, &*e, &[("chain", chain.as_ref())]),
            }
        }
        Ok(())
    }

    async fn update_validators_for_chain(&mut self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let validators = self.chain_providers.get_validators(chain).await?;
        let addresses: Vec<_> = validators.into_iter().filter_map(|v| v.as_scan_address(chain)).collect();
        let count = addresses.len();
        self.database.add_scan_addresses(addresses)?;
        Ok(count)
    }

    pub async fn update_validators_from_static_assets(&mut self, name: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chains = vec![Chain::Tron, Chain::SmartChain];
        let static_assets_client = StaticAssetsClient::new(&self.assets_url);

        for chain in chains {
            match self.update_validators_from_static_assets_for_chain(chain, &static_assets_client).await {
                Ok(count) => info_with_context(name, &[("chain", chain.as_ref()), ("count", &count.to_string())]),
                Err(e) => error_with_context(name, &*e, &[("chain", chain.as_ref())]),
            }
        }
        Ok(())
    }

    async fn update_validators_from_static_assets_for_chain(
        &mut self,
        chain: Chain,
        static_assets_client: &StaticAssetsClient,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let static_validators = static_assets_client.get_validators(chain).await?;
        let validators: Vec<_> = static_validators.into_iter().map(|v| StakeValidator::new(v.id, v.name)).collect();
        let addresses: Vec<_> = validators.into_iter().filter_map(|v| v.as_scan_address(chain)).collect();
        let count = addresses.len();
        self.database.add_scan_addresses(addresses)?;
        Ok(count)
    }
}

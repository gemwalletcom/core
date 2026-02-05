use api_connector::StaticAssetsClient;
use primitives::{Chain, StakeValidator};
use settings_chain::ChainProviders;
use std::error::Error;
use std::sync::Arc;
use storage::{Database, ScanAddressesRepository};

pub struct ValidatorScanner {
    chain_providers: Arc<ChainProviders>,
    database: Database,
}

impl ValidatorScanner {
    pub fn new(chain_providers: Arc<ChainProviders>, database: Database) -> Self {
        Self { chain_providers, database }
    }

    pub async fn update_validators_for_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let validators = self.chain_providers.get_validators(chain).await?;
        let addresses: Vec<_> = validators.into_iter().filter_map(|v| v.as_scan_address(chain)).collect();
        let count = addresses.len();
        self.database.scan_addresses()?.add_scan_addresses(addresses)?;
        Ok(count)
    }

    pub async fn update_validators_from_static_assets_for_chain(&self, chain: Chain, assets_url: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let static_assets_client = StaticAssetsClient::new(assets_url);
        let static_validators = static_assets_client.get_validators(chain).await?;
        let validators: Vec<_> = static_validators.into_iter().map(|v| StakeValidator::new(v.id, v.name)).collect();
        let addresses: Vec<_> = validators.into_iter().filter_map(|v| v.as_scan_address(chain)).collect();
        let count = addresses.len();
        self.database.scan_addresses()?.add_scan_addresses(addresses)?;
        Ok(count)
    }
}

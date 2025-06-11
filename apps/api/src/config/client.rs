use primitives::{ConfigResponse, ConfigVersions, PlatformStore, Release, SwapConfig, SwapProvider};
use std::{error::Error, str::FromStr};
use storage::DatabaseClient;

pub struct ConfigClient {
    database: DatabaseClient,
}

impl ConfigClient {
    pub async fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub fn get_config(&mut self) -> Result<ConfigResponse, Box<dyn Error>> {
        let fiat_on_ramp_assets = self.database.get_assets_is_buyable()?.len() as i32;
        let fiat_off_ramp_assets = self.database.get_assets_is_sellable()?.len() as i32;
        let swap_assets_version = self.database.get_swap_assets_version()?;
        let releases = self.database.get_releases()?;

        let releases = releases
            .into_iter()
            .map(|x| Release {
                store: PlatformStore::from_str(&x.platform_store).unwrap(),
                version: x.version,
                upgrade_required: x.upgrade_required,
            })
            .collect();

        let response = ConfigResponse {
            releases,
            versions: ConfigVersions {
                fiat_on_ramp_assets,
                fiat_off_ramp_assets,
                swap_assets: swap_assets_version,
            },
            swap: SwapConfig {
                enabled_providers: SwapProvider::all(),
            },
        };
        Ok(response)
    }
}

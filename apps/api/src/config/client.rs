use primitives::{ConfigResponse, ConfigVersions, PlatformStore, Release, SwapConfig, SwapProvider};
use std::{error::Error, str::FromStr};
use storage::{AssetFilter, Database};

#[derive(Clone)]
pub struct ConfigClient {
    database: Database,
}

impl ConfigClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_config(&self) -> Result<ConfigResponse, Box<dyn Error + Send + Sync>> {
        let fiat_on_ramp_assets = self.database.client()?.assets().get_assets_by_filter(vec![AssetFilter::IsBuyable(true)])?.len() as i32;
        let fiat_off_ramp_assets = self
            .database
            .client()?
            .assets()
            .get_assets_by_filter(vec![AssetFilter::IsSellable(true)])?
            .len() as i32;
        let swap_assets_version = self.database.client()?.assets().get_swap_assets_version()?;
        let releases = self.database.client()?.releases().get_releases()?;

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
                enabled_providers: SwapProvider::all().iter().map(|x| x.as_ref().to_string()).collect(),
            },
        };
        Ok(response)
    }
}

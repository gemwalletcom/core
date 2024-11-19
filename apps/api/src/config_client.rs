use primitives::{
    config::{ConfigAndroidApp, ConfigApp, ConfigAppVersion, ConfigIOSApp, ConfigResponse, ConfigVersions, Release},
    PlatformStore,
};
use std::{error::Error, str::FromStr};
use storage::DatabaseClient;

pub struct Client {
    database: DatabaseClient,
}

impl Client {
    pub async fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub fn get_config(&mut self) -> Result<ConfigResponse, Box<dyn Error>> {
        let fiat_on_ramp_assets = self.database.get_fiat_assets_is_buyable()?.len() as i32;
        let fiat_off_ramp_assets = self.database.get_fiat_assets_is_sellable()?.len() as i32;

        let swap_assets_version = self.database.get_swap_assets_version()?;

        let releases = self.database.get_releases()?;

        //TODO: Remove later
        let ios = "1.3.48".to_string(); //versions.first().expect("expect ios to be");
        let android = "1.2.171".to_string(); //versions.last().expect("expect android to be last");

        let app: ConfigApp = ConfigApp {
            ios: ConfigIOSApp {
                version: ConfigAppVersion {
                    production: ios.clone(),
                    beta: ios.clone(),
                    alpha: ios.clone(),
                },
            },
            android: ConfigAndroidApp {
                version: ConfigAppVersion {
                    production: android.clone(),
                    beta: android.clone(),
                    alpha: android.clone(),
                },
            },
        };
        let releases = releases
            .into_iter()
            .map(|x| Release {
                store: PlatformStore::from_str(&x.platform_store).unwrap(),
                version: x.version,
                upgrade_required: x.upgrade_required,
            })
            .collect();

        let response: ConfigResponse = ConfigResponse {
            app,
            releases,
            versions: ConfigVersions {
                fiat_assets: fiat_on_ramp_assets,
                fiat_on_ramp_assets,
                fiat_off_ramp_assets,
                swap_assets: swap_assets_version,
            },
        };
        Ok(response)
    }
}

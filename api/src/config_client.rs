use primitives::{
    config::{
        ConfigAndroidApp, ConfigApp, ConfigAppVersion, ConfigIOSApp, ConfigResponse, ConfigVersions,
    },
    tokenlist::TokenListChainVersion,
};
use std::error::Error;
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
        let nodes_version = self.database.get_nodes_version()?;
        let fiat_assets_version = self.database.get_fiat_assets_version()?;
        let swap_assets_version = self.database.get_swap_assets_version()?;

        let versions = self.database.get_versions()?;
        let ios = versions.first().expect("expect ios to be");
        let android = versions.last().expect("expect android to be last");
        let app: ConfigApp = ConfigApp {
            ios: ConfigIOSApp {
                version: ConfigAppVersion {
                    production: ios.production.clone(),
                    beta: ios.beta.clone(),
                    alpha: ios.alpha.clone(),
                },
            },
            android: ConfigAndroidApp {
                version: ConfigAppVersion {
                    production: android.production.clone(),
                    beta: android.beta.clone(),
                    alpha: android.alpha.clone(),
                },
            },
        };
        let response: ConfigResponse = ConfigResponse {
            app,
            versions: ConfigVersions {
                nodes: nodes_version,
                fiat_assets: fiat_assets_version,
                swap_assets: swap_assets_version,
            },
        };
        Ok(response)
    }
}

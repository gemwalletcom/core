mod assets_index_updater;
mod nfts_index_updater;
mod perpetuals_index_updater;

use assets_index_updater::AssetsIndexUpdater;
use job_runner::{ShutdownReceiver, run_job};
use nfts_index_updater::NftsIndexUpdater;
use perpetuals_index_updater::PerpetualsIndexUpdater;
use primitives::ConfigKey;
use search_index::SearchIndexClient;
use settings::Settings;
use std::error::Error;
use storage::ConfigCacher;
use tokio::task::JoinHandle;

pub async fn jobs(settings: Settings, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());
    let config = ConfigCacher::new(database.clone());

    let assets_update_interval = config.get_duration(ConfigKey::SearchAssetsUpdateInterval)?;
    let perpetuals_update_interval = config.get_duration(ConfigKey::SearchPerpetualsUpdateInterval)?;
    let nfts_update_interval = config.get_duration(ConfigKey::SearchNftsUpdateInterval)?;

    let assets_index_updater = tokio::spawn(run_job("Update assets index", assets_update_interval, shutdown_rx.clone(), {
        let database = database.clone();
        let search_index_client = search_index_client.clone();

        move || {
            let updater = AssetsIndexUpdater::new(database.clone(), &search_index_client);
            async move { updater.update().await }
        }
    }));

    let perpetuals_index_updater = tokio::spawn(run_job("Update perpetuals index", perpetuals_update_interval, shutdown_rx.clone(), {
        let database = database.clone();
        let search_index_client = search_index_client.clone();

        move || {
            let updater = PerpetualsIndexUpdater::new(database.clone(), &search_index_client);
            async move { updater.update().await }
        }
    }));

    let nfts_index_updater = tokio::spawn(run_job("Update NFTs index", nfts_update_interval, shutdown_rx, {
        let database = database.clone();
        let search_index_client = search_index_client.clone();

        move || {
            let updater = NftsIndexUpdater::new(database.clone(), &search_index_client);
            async move { updater.update().await }
        }
    }));

    Ok(vec![assets_index_updater, perpetuals_index_updater, nfts_index_updater])
}

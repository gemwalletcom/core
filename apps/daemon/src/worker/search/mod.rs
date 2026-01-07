mod assets_index_updater;
mod nfts_index_updater;
mod perpetuals_index_updater;

use assets_index_updater::AssetsIndexUpdater;
use job_runner::run_job;
use nfts_index_updater::NftsIndexUpdater;
use perpetuals_index_updater::PerpetualsIndexUpdater;
use primitives::ConfigKey;
use search_index::SearchIndexClient;
use settings::Settings;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use storage::ConfigRepository;

pub async fn jobs(settings: Settings) -> Result<Vec<Pin<Box<dyn Future<Output = ()> + Send>>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());

    let assets_update_interval = database.config()?.get_config_duration(ConfigKey::SearchAssetsUpdateInterval)?;
    let perpetuals_update_interval = database.config()?.get_config_duration(ConfigKey::SearchPerpetualsUpdateInterval)?;
    let nfts_update_interval = database.config()?.get_config_duration(ConfigKey::SearchNftsUpdateInterval)?;

    let assets_index_updater = run_job("Update assets index", assets_update_interval, {
        let database = database.clone();
        let search_index_client = search_index_client.clone();

        move || {
            let updater = AssetsIndexUpdater::new(database.clone(), &search_index_client);
            async move { updater.update().await }
        }
    });

    let perpetuals_index_updater = run_job("Update perpetuals index", perpetuals_update_interval, {
        let database = database.clone();
        let search_index_client = search_index_client.clone();

        move || {
            let updater = PerpetualsIndexUpdater::new(database.clone(), &search_index_client);
            async move { updater.update().await }
        }
    });

    let nfts_index_updater = run_job("Update NFTs index", nfts_update_interval, {
        let database = database.clone();
        let search_index_client = search_index_client.clone();

        move || {
            let updater = NftsIndexUpdater::new(database.clone(), &search_index_client);
            async move { updater.update().await }
        }
    });

    Ok(vec![
        Box::pin(assets_index_updater),
        Box::pin(perpetuals_index_updater),
        Box::pin(nfts_index_updater),
    ])
}

mod assets_index_updater;
mod nfts_index_updater;
mod perpetuals_index_updater;

use assets_index_updater::AssetsIndexUpdater;
use job_runner::run_job;
use nfts_index_updater::NftsIndexUpdater;
use perpetuals_index_updater::PerpetualsIndexUpdater;
use search_index::SearchIndexClient;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());

    let assets_index_updater = run_job("Update assets index", Duration::from_secs(settings.daemon.search.assets_update_interval), {
        let database = database.clone();
        let search_index_client = search_index_client.clone();

        move || {
            let mut updater = AssetsIndexUpdater::new(database.clone(), &search_index_client);
            async move { updater.update().await }
        }
    });

    let perpetuals_index_updater = run_job("Update perpetuals index", Duration::from_secs(settings.daemon.search.assets_update_interval), {
        let database = database.clone();
        let search_index_client = search_index_client.clone();

        move || {
            let mut updater = PerpetualsIndexUpdater::new(database.clone(), &search_index_client);
            async move { updater.update().await }
        }
    });

    let nfts_index_updater = run_job("Update NFTs index", Duration::from_secs(settings.daemon.search.assets_update_interval), {
        let database = database.clone();
        let search_index_client = search_index_client.clone();

        move || {
            let mut updater = NftsIndexUpdater::new(database.clone(), &search_index_client);
            async move { updater.update().await }
        }
    });

    vec![Box::pin(assets_index_updater), Box::pin(perpetuals_index_updater), Box::pin(nfts_index_updater)]
}

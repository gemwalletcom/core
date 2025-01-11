mod assets_index_updater;

use assets_index_updater::AssetsIndexUpdater;
use job_runner::run_job;
use search_index::SearchIndexClient;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());

    let assets_index_updater = run_job("Update assets index", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        let search_index_client = search_index_client.clone();

        move || {
            let mut updater = AssetsIndexUpdater::new(&settings.postgres.url, &search_index_client);
            async move { updater.update().await }
        }
    });

    vec![Box::pin(assets_index_updater)]
}

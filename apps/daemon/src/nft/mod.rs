mod opensea_updater;
use opensea_updater::OpenSeaUpdater;

use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let open_sea_collections_updater = run_job("Update OpenSea collections", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());

        move || {
            let mut updater = OpenSeaUpdater::new(&settings.postgres.url);
            async move { updater.update().await }
        }
    });

    vec![Box::pin(open_sea_collections_updater)]
}

use std::sync::Arc;

use api_connector::{AppStoreClient, GooglePlayClient};
use storage::DatabaseClient;
use tokio::task;
use tokio::time::{Duration, Instant};

pub mod appstore_updater;
pub use appstore_updater::AppstoreUpdater;
pub mod googleplay_updater;
pub use googleplay_updater::GooglePlayUpdater;

#[tokio::main]
pub async fn main() {
    println!("operator init");

    let settings = Arc::new(settings::Settings::new().unwrap());
    let settings_operator = Arc::new(settings::SettingsOperator::new().unwrap());

    let appstore = Arc::new(settings_operator.appstore.clone());
    let googleplay = Arc::new(settings_operator.googleplay.clone());

    let google_play_update_details_job = task::spawn(run_job("Google Play. Update datails", Duration::from_millis(googleplay.refresh_interval_ms), {
        let settings = Arc::new(settings.clone());
        let appstore = Arc::new(settings_operator.appstore.clone());
        let googleplay = Arc::new(settings_operator.googleplay.clone());
        move || {
            let settings = Arc::clone(&settings);
            let appstore = Arc::clone(&appstore);
            let googleplay = Arc::clone(&googleplay);

            async move {
                let googleplay_client = GooglePlayClient::new(googleplay.url.clone());
                let mut googleplay_updater = GooglePlayUpdater::new(googleplay_client, DatabaseClient::new(&settings.postgres.url), googleplay.timeout_ms);

                googleplay_updater.update_details(appstore.apps.clone(), appstore.languages.clone()).await;
            }
        }
    }));

    let google_play_update_positions_job = task::spawn(run_job(
        "Google Play. Update positions",
        Duration::from_millis(googleplay.refresh_interval_ms),
        {
            let settings = Arc::new(settings.clone());
            let appstore = Arc::new(settings_operator.appstore.clone());
            let googleplay = Arc::new(settings_operator.googleplay.clone());

            move || {
                let settings = Arc::clone(&settings);
                let appstore = Arc::clone(&appstore);
                let googleplay = Arc::clone(&googleplay);

                async move {
                    let googleplay_client = GooglePlayClient::new(googleplay.url.clone());
                    let mut googleplay_updater = GooglePlayUpdater::new(googleplay_client, DatabaseClient::new(&settings.postgres.url), googleplay.timeout_ms);

                    googleplay_updater
                        .update_positions(appstore.keys.clone(), appstore.apps.clone(), appstore.languages.clone())
                        .await;
                }
            }
        },
    ));
    let google_play_update_reviews_job = task::spawn(run_job(
        "Google Play. Update positions",
        Duration::from_millis(googleplay.refresh_interval_ms),
        {
            let settings = Arc::new(settings.clone());
            let appstore = Arc::new(settings_operator.appstore.clone());
            let googleplay = Arc::new(settings_operator.googleplay.clone());

            move || {
                let settings = Arc::clone(&settings);
                let appstore = Arc::clone(&appstore);
                let googleplay = Arc::clone(&googleplay);

                async move {
                    let googleplay_client = GooglePlayClient::new(googleplay.url.clone());
                    let mut googleplay_updater = GooglePlayUpdater::new(googleplay_client, DatabaseClient::new(&settings.postgres.url), googleplay.timeout_ms);

                    googleplay_updater.update_reviews(appstore.apps.clone(), appstore.languages.clone()).await;
                }
            }
        },
    ));

    let app_store_update_details_job = task::spawn(run_job("App Store. Update datails", Duration::from_millis(appstore.refresh_interval_ms), {
        let settings = Arc::new(settings.clone());
        let appstore = Arc::new(settings_operator.appstore.clone());
        move || {
            let settings = Arc::clone(&settings);
            let appstore = Arc::clone(&appstore);

            async move {
                let mut appstore_updater = AppstoreUpdater::new(AppStoreClient::new(), DatabaseClient::new(&settings.postgres.url), appstore.timeout_ms);

                appstore_updater.update_details(appstore.apps.clone(), appstore.languages.clone()).await;
            }
        }
    }));

    let app_store_update_positions_job = task::spawn(run_job("App Store. Update positions", Duration::from_millis(appstore.refresh_interval_ms), {
        let settings = Arc::new(settings.clone());
        let appstore = Arc::new(settings_operator.appstore.clone());

        move || {
            let settings = Arc::clone(&settings);
            let appstore = Arc::clone(&appstore);

            async move {
                let mut appstore_updater = AppstoreUpdater::new(AppStoreClient::new(), DatabaseClient::new(&settings.postgres.url), appstore.timeout_ms);

                appstore_updater
                    .update_positions(appstore.keys.clone(), appstore.apps.clone(), appstore.languages.clone())
                    .await;
            }
        }
    }));
    let app_store_update_reviews_job = task::spawn(run_job("App Store. Update reviews", Duration::from_millis(appstore.refresh_interval_ms), {
        let settings = Arc::new(settings.clone());
        let appstore = Arc::new(settings_operator.appstore.clone());

        move || {
            let settings = Arc::clone(&settings);
            let appstore = Arc::clone(&appstore);

            async move {
                let mut appstore_updater = AppstoreUpdater::new(AppStoreClient::new(), DatabaseClient::new(&settings.postgres.url), appstore.timeout_ms);

                appstore_updater.update_reviews(appstore.apps.clone(), appstore.languages.clone()).await;
            }
        }
    }));

    let _ = tokio::join!(
        app_store_update_details_job,
        app_store_update_positions_job,
        app_store_update_reviews_job,
        google_play_update_details_job,
        google_play_update_positions_job,
        google_play_update_reviews_job
    );
}

async fn run_job<F, Fut>(name: &'static str, interval_duration: Duration, job_fn: F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let start = Instant::now();
    let mut interval = tokio::time::interval_at(start, interval_duration);

    loop {
        interval.tick().await;
        println!("Start job {}...", name);

        // Run the async task with a timeout, but without expecting any output
        let result = tokio::time::timeout(interval_duration, job_fn()).await;
        match result {
            Ok(_) => println!("{} finished successfully", name),
            Err(_) => println!("{} timed out.", name),
        }
    }
}

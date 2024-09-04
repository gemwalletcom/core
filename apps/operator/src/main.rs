use api_connector::AppStoreClient;
use storage::DatabaseClient;

pub mod appstore_updater;
pub use appstore_updater::AppstoreUpdater;

#[tokio::main]
pub async fn main() {
    println!("operator init");

    let settings = settings::Settings::new().unwrap();
    let settings_operator = settings::SettingsOperator::new().unwrap();

    let keys = settings_operator.appstore.keys;
    let apps = settings_operator.appstore.apps;
    let languages = settings_operator.appstore.languages;
    let client = AppStoreClient::new();

    let database = DatabaseClient::new(&settings.postgres.url.clone());
    let mut appstore_updater = AppstoreUpdater::new(client, database, settings_operator.appstore.timeout_ms);

    loop {
        appstore_updater.update_details(apps.clone(), languages.clone()).await;

        appstore_updater.update_positions(keys.clone(), apps.clone(), languages.clone()).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(settings_operator.appstore.refresh_interval_ms)).await;
    }
}

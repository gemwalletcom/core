use api_connector::AppStoreClient;
use storage::ClickhouseDatabase;

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
    let clickhouse_database = ClickhouseDatabase::new(&settings.clickhouse.url);
    let appstore_updater = AppstoreUpdater::new(client, clickhouse_database);

    loop {
        appstore_updater.update_positions(keys.clone(), apps.clone(), languages.clone()).await;

        appstore_updater.update_details(apps.clone(), languages.clone()).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await;
    }
}

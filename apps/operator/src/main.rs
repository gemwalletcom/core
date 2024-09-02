use api_connector::AppStoreClient;
use storage::ClickhouseDatabase;

pub mod appstore_positions;
pub use appstore_positions::AppstorPositionsUpdater;

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
    let appstore_positions_updater = AppstorPositionsUpdater::new(client, clickhouse_database);

    loop {
        appstore_positions_updater.update(keys.clone(), apps.clone(), languages.clone()).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await;
    }
}

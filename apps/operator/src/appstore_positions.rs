use api_connector::AppStoreClient;
use chrono::prelude::*;
use settings::{OperatorAppStoreApp, OperatorAppStoreLanguage};
use storage::{models::chart::Position, ClickhouseDatabase};

pub struct AppstorPositionsUpdater {
    client: AppStoreClient,
    clickhouse_database: ClickhouseDatabase,
}

impl AppstorPositionsUpdater {
    pub fn new(client: AppStoreClient, clickhouse_database: ClickhouseDatabase) -> Self {
        Self { client, clickhouse_database }
    }

    pub async fn update(&self, keys: Vec<String>, apps: Vec<OperatorAppStoreApp>, languages: Vec<OperatorAppStoreLanguage>) {
        for key in keys {
            let mut positions: Vec<Position> = Vec::new();
            let ts = Utc::now().timestamp();

            println!("Update key: {}", key);

            for language in languages.clone() {
                match self.client.search_apps(key.as_str(), &language.code, 200).await {
                    Ok(response) => {
                        println!("Found key: {}, language: {}, results: {}", key, language.code, response.results.len());

                        for (position, result) in response.results.iter().enumerate() {
                            match apps.clone().into_iter().find(|a| a.id == result.track_id) {
                                Some(app) => {
                                    let position = Position {
                                        store: "appstore".to_string(),
                                        app: app.name.to_string(),
                                        keyword: key.to_string(),
                                        country: language.name.to_string(),
                                        position: (position + 1) as u32,
                                        ts: (ts / 86400) as u16,
                                    };

                                    positions.push(position)
                                }
                                _ => (),
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch apps, keyword: {}, language: {}: {}", key, language.code, e);
                    }
                }
            }

            match self.clickhouse_database.add_positions(positions.clone()).await {
                Ok(_) => {
                    //println!("Inserted {} positions", positions.len());
                }
                Err(e) => {
                    eprintln!("Failed to insert positions: {}", e);
                }
            }
        }
    }
}

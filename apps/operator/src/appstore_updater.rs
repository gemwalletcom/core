use api_connector::AppStoreClient;
use settings::{OperatorAppStoreApp, OperatorAppStoreLanguage};
use storage::{
    models::operator::{AppStoreInformation, AppStorePosition},
    DatabaseClient,
};

pub struct AppstoreUpdater {
    client: AppStoreClient,
    database: DatabaseClient,
}

impl AppstoreUpdater {
    pub fn new(client: AppStoreClient, database: DatabaseClient) -> Self {
        Self { client, database }
    }

    pub async fn update_details(&mut self, apps: Vec<OperatorAppStoreApp>, languages: Vec<OperatorAppStoreLanguage>) {
        for app in apps {
            let mut values: Vec<AppStoreInformation> = Vec::new();

            for language in languages.clone() {
                match self.client.lookup(app.id, &language.code).await {
                    Ok(response) => {
                        let information = AppStoreInformation {
                            store: "appstore".to_string(),
                            app: app.name.to_string(),
                            country: language.name.to_string(),
                            title: response.track_name,
                            ratings: response.user_rating_count,
                            average_rating: response.average_user_rating,
                        };
                        values.push(information)
                    }
                    Err(e) => {
                        eprintln!("Failed to look up app {}, language: {}, {:?}", app.name, language.code, e);
                    }
                }
            }

            match self.database.add_appstore_information(values.clone()) {
                Ok(_) => {
                    //println!("Inserted {} positions", positions.len());
                }
                Err(e) => {
                    eprintln!("Failed to insert appstore information: {}", e);
                }
            }
        }
    }

    pub async fn update_positions(&mut self, keys: Vec<String>, apps: Vec<OperatorAppStoreApp>, languages: Vec<OperatorAppStoreLanguage>) {
        for key in keys {
            let mut positions: Vec<AppStorePosition> = Vec::new();

            println!("Update key: {}", key);

            for language in languages.clone() {
                match self.client.search_apps(key.as_str(), &language.code, 200).await {
                    Ok(response) => {
                        println!("Found key: {}, language: {}, results: {}", key, language.code, response.results.len());

                        for (position, result) in response.results.iter().enumerate() {
                            match apps.clone().into_iter().find(|a| a.id == result.track_id) {
                                Some(app) => {
                                    let position = AppStorePosition {
                                        store: "appstore".to_string(),
                                        app: app.name.to_string(),
                                        keyword: key.to_string(),
                                        country: language.name.to_string(),
                                        position: (position + 1) as i32,
                                    };

                                    positions.push(position)
                                }
                                _ => (),
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch apps, keyword: {}, language: {}: {:?}", key, language.code, e);
                    }
                }
            }

            match self.database.add_appstore_positions(positions.clone()) {
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

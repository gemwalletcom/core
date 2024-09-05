use api_connector::{
    app_store_client::models::{AppStoreReviewEntries, AppStoreReviewEntry},
    AppStoreClient,
};
use settings::{OperatorAppStoreApp, OperatorAppStoreLanguage};
use storage::{
    models::operator::{AppStoreInformation, AppStorePosition, AppStoreReview},
    DatabaseClient,
};

pub struct AppstoreUpdater {
    client: AppStoreClient,
    database: DatabaseClient,
    timeout_ms: u64,
}

impl AppstoreUpdater {
    pub fn new(client: AppStoreClient, database: DatabaseClient, timeout_ms: u64) -> Self {
        Self { client, database, timeout_ms }
    }

    pub async fn update_details(&mut self, apps: Vec<OperatorAppStoreApp>, languages: Vec<OperatorAppStoreLanguage>) {
        for app in apps {
            let mut values: Vec<AppStoreInformation> = Vec::new();

            for language in languages.clone() {
                println!("Update details. Start for language: {}", language.code);

                match self.client.lookup(app.id, &language.code).await {
                    Ok(response) => {
                        let information = AppStoreInformation {
                            store: "appstore".to_string(),
                            app: app.name.to_string(),
                            country: language.name.to_string(),
                            title: response.track_name,
                            version: response.version,
                            ratings: response.user_rating_count,
                            average_rating: response.average_user_rating,
                        };
                        values.push(information);

                        println!("Update details. Found app: {}, language: {}", app.name, language.code);
                    }
                    Err(e) => {
                        eprintln!("Update details. Failed to look up app {}, language: {}, {:?}", app.name, language.code, e);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(self.timeout_ms)).await;
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

            for language in languages.clone() {
                println!("Update positions. Start for language: {}, key: {}", language.code, key);

                match self.client.search_apps(key.as_str(), &language.code, 200).await {
                    Ok(response) => {
                        for (position, result) in response.results.iter().enumerate() {
                            if let Some(app) = apps.clone().into_iter().find(|a| a.id == result.track_id) {
                                let position = (position + 1) as i32;
                                positions.push(AppStorePosition {
                                    store: "appstore".to_string(),
                                    app: app.name.to_string(),
                                    keyword: key.to_string(),
                                    country: language.name.to_string(),
                                    position,
                                });

                                println!(
                                    "Update positions. Found app: {}, language: {}, key: {}, position: {}",
                                    app.name, language.code, key, position
                                );
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(self.timeout_ms)).await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Update positions. Failed to fetch apps, keyword: {}, language: {}: {:?}", key, language.code, e);
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

    pub async fn update_reviews(&mut self, apps: Vec<OperatorAppStoreApp>, languages: Vec<OperatorAppStoreLanguage>) {
        for app in apps {
            let mut values: Vec<AppStoreReview> = Vec::new();

            for language in languages.clone() {
                println!("Update reviews. Start for language: {}", language.code);

                match self.client.reviews(app.id, &language.code).await {
                    Ok(response) => {
                        if let Some(entry) = response.feed.entry {
                            match entry {
                                AppStoreReviewEntries::Single(review) => values.push(Self::review(app.name.clone(), language.name.clone(), review)),
                                AppStoreReviewEntries::Multiple(reviews) => {
                                    for review in reviews {
                                        values.push(Self::review(app.name.clone(), language.name.clone(), review))
                                    }
                                }
                            }
                        }

                        println!("Update reviews. Found app: {}, language: {}", app.name, language.code);
                    }
                    Err(e) => {
                        eprintln!("Update reviews. Failed to look up app {}, language: {}, {:?}", app.name, language.code, e);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(self.timeout_ms)).await;
            }

            match self.database.add_appstore_reviews(values.clone()) {
                Ok(_) => {
                    //println!("Inserted {} appstore reviews", positions.len());
                }
                Err(e) => {
                    eprintln!("Failed to insert appstore reviews: {}", e);
                }
            }
        }
    }

    fn review(app_name: String, country: String, review: AppStoreReviewEntry) -> AppStoreReview {
        AppStoreReview {
            store: "appstore".to_string(),
            app: app_name.to_string(),
            country: country.to_string(),
            title: review.title.label,
            version: review.version.label,
            review_id: review.id.label,
            content: review.content.label,
            author: review.author.name.label,
            rating: review.rating.label.parse::<i32>().unwrap_or(0),
        }
    }
}

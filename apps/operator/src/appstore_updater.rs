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
                match self.client.lookup(app.id, &language.country_code).await {
                    Ok(response) => {
                        let information = AppStoreInformation {
                            store: "appstore".to_string(),
                            app: app.name.to_string(),
                            country: language.country.to_string(),
                            title: response.track_name,
                            version: response.version,
                            ratings: response.user_rating_count,
                            average_rating: response.average_user_rating,
                            release_date: response.release_date.naive_utc(),
                            current_version_release_date: response.current_version_release_date.naive_utc(),
                        };
                        values.push(information.clone());
                    }
                    Err(e) => {
                        eprintln!(
                            "App Store. Update details. Failed to look up app {}, language: {}, {:?}",
                            app.name, language.country_code, e
                        );
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
                match self.client.search_apps(key.as_str(), &language.country_code, 200).await {
                    Ok(response) => {
                        for (position, result) in response.results.iter().enumerate() {
                            if let Some(app) = apps.clone().into_iter().find(|a| a.id == result.track_id) {
                                let position = (position + 1) as i32;
                                positions.push(AppStorePosition {
                                    store: "appstore".to_string(),
                                    app: app.name.to_string(),
                                    keyword: key.to_string(),
                                    country: language.country.to_string(),
                                    position,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Update positions. Failed to fetch apps, keyword: {}, language: {}: {:?}", key, language.country_code, e);
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

            tokio::time::sleep(tokio::time::Duration::from_millis(self.timeout_ms)).await;
        }
    }

    pub async fn update_reviews(&mut self, apps: Vec<OperatorAppStoreApp>, languages: Vec<OperatorAppStoreLanguage>) {
        for app in apps {
            let mut values: Vec<AppStoreReview> = Vec::new();
            for language in languages.clone() {
                match self.client.reviews(app.id, &language.country_code).await {
                    Ok(response) => {
                        if let Some(entry) = response.feed.entry {
                            match entry {
                                AppStoreReviewEntries::Single(review) => values.push(Self::review(app.name.clone(), language.country.clone(), review)),
                                AppStoreReviewEntries::Multiple(reviews) => {
                                    for review in reviews {
                                        values.push(Self::review(app.name.clone(), language.country.clone(), review))
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Update reviews. Failed to look up app {}, language: {}, {:?}", app.name, language.country_code, e);
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

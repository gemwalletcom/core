use api_connector::{google_play_client::models::Review, GooglePlayClient};
use settings::{OperatorAppStoreApp, OperatorAppStoreLanguage};
use storage::{
    models::operator::{AppStoreInformation, AppStorePosition, AppStoreReview},
    DatabaseClient,
};

pub struct GooglePlayUpdater {
    client: GooglePlayClient,
    database: DatabaseClient,
    timeout_ms: u64,
}

impl GooglePlayUpdater {
    pub fn new(client: GooglePlayClient, database: DatabaseClient, timeout_ms: u64) -> Self {
        Self { client, database, timeout_ms }
    }

    pub async fn update_details(&mut self, apps: Vec<OperatorAppStoreApp>, languages: Vec<OperatorAppStoreLanguage>) {
        for app in apps {
            let mut values: Vec<AppStoreInformation> = Vec::new();

            for language in languages.clone() {
                match self
                    .client
                    .lookup(app.package_id.clone(), &language.country_code, &language.language_code)
                    .await
                {
                    Ok(response) => {
                        let information = AppStoreInformation {
                            store: "googleplay".to_string(),
                            app: app.name.to_string(),
                            country: language.country.to_string(),
                            title: response.title.clone(),
                            version: response.version.clone(),
                            ratings: Some(response.ratings),
                            average_rating: Some(response.score.unwrap_or_default()),
                            release_date: response.release_date().naive_utc(),
                            current_version_release_date: response.updated_date().naive_utc(),
                        };
                        values.push(information.clone());
                    }
                    Err(e) => {
                        eprintln!(
                            "Google Play. Update details. Failed to look up app {}, language: {}, {:?}",
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
                    eprintln!("Google Play. Failed to insert appstore information: {}", e);
                }
            }
        }
    }

    pub async fn update_positions(&mut self, keys: Vec<String>, apps: Vec<OperatorAppStoreApp>, languages: Vec<OperatorAppStoreLanguage>) {
        for key in keys {
            let mut positions: Vec<AppStorePosition> = Vec::new();
            for language in languages.clone() {
                match self
                    .client
                    .search_apps(key.as_str(), &language.country_code, &language.language_code, 250)
                    .await
                {
                    Ok(response) => {
                        for (position, result) in response.iter().enumerate() {
                            if let Some(app) = apps.clone().into_iter().find(|a| a.package_id == result.app_id) {
                                let position = (position + 1) as i32;
                                positions.push(AppStorePosition {
                                    store: "googleplay".to_string(),
                                    app: app.name.to_string(),
                                    keyword: key.to_string(),
                                    country: language.country.to_string(),
                                    position,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Google Play. Update positions. Failed to fetch apps, keyword: {}, language: {}: {:?}",
                            key, language.country_code, e
                        );
                    }
                }
            }

            match self.database.add_appstore_positions(positions.clone()) {
                Ok(_) => {
                    //println!("Inserted {} positions", positions.len());
                }
                Err(e) => {
                    eprintln!("Google Play. Failed to insert positions: {}", e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(self.timeout_ms)).await;
        }
    }

    pub async fn update_reviews(&mut self, apps: Vec<OperatorAppStoreApp>, languages: Vec<OperatorAppStoreLanguage>) {
        for app in apps {
            let mut values: Vec<AppStoreReview> = Vec::new();

            for language in languages.clone() {
                match self
                    .client
                    .reviews(app.package_id.as_str(), &language.country_code, &language.language_code)
                    .await
                {
                    Ok(response) => {
                        for review in response {
                            values.push(Self::review(app.name.clone(), language.country.clone(), review))
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Google Play. Update reviews. Failed to look up app {}, language: {}, {:?}",
                            app.name, language.language_code, e
                        );
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(self.timeout_ms)).await;
            }

            match self.database.add_appstore_reviews(values.clone()) {
                Ok(_) => {
                    //println!("Inserted {} appstore reviews", positions.len());
                }
                Err(e) => {
                    eprintln!("Google Play. Failed to insert appstore reviews: {}", e);
                }
            }
        }
    }

    fn review(app_name: String, country: String, review: Review) -> AppStoreReview {
        AppStoreReview {
            store: "googleplay".to_string(),
            app: app_name.to_string(),
            country: country.to_string(),
            title: "".to_string(),
            version: review.version.unwrap_or_default(),
            review_id: review.id,
            content: review.text,
            author: review.user_name,
            rating: review.score.unwrap_or_default() as i32,
        }
    }
}

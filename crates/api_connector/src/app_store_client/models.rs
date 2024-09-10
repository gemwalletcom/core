use chrono::{DateTime, Utc};
use serde::Deserialize;
#[derive(Debug)]
pub enum AppStoreError {
    Request(reqwest::Error),
    AppNotFound,
}

impl From<reqwest::Error> for AppStoreError {
    fn from(err: reqwest::Error) -> AppStoreError {
        AppStoreError::Request(err)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStoreResponse {
    pub results: Vec<App>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub track_id: u64,
    pub version: String,
    pub user_rating_count: Option<f64>,
    pub average_user_rating: Option<f64>,
    pub track_name: String,
    pub release_date: DateTime<Utc>,
    pub current_version_release_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStoreReviews {
    pub feed: AppStoreFeed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStoreFeed {
    pub entry: Option<AppStoreReviewEntries>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AppStoreReviewEntries {
    Single(AppStoreReviewEntry),
    Multiple(Vec<AppStoreReviewEntry>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStoreReviewEntry {
    #[serde(rename = "im:rating")]
    pub rating: AppStoreReviewLabel,
    #[serde(rename = "im:version")]
    pub version: AppStoreReviewLabel,
    pub id: AppStoreReviewLabel,
    pub title: AppStoreReviewLabel,
    pub content: AppStoreReviewLabel,
    pub author: AppStoreReviewAuthor,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStoreReviewLabel {
    pub label: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStoreReviewAuthor {
    pub name: AppStoreReviewLabel,
}

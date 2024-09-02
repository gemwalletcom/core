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
    pub version: Option<String>,
    pub user_rating_count: Option<f64>,
    pub average_user_rating: Option<f64>,
}

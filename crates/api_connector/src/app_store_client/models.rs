use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStoreResponse {
    pub results: Vec<App>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub track_id: u64,
    pub description: Option<String>,
    pub version: Option<String>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ITunesLookupResponse {
    pub results: Vec<ITunesLoopUpResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ITunesLoopUpResult {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<GitHubRepositoryAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepositoryAsset {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamsungStoreDetail {
    #[serde(rename = "DetailMain")]
    pub details: SamsungStoreDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamsungStoreDetails {
    #[serde(rename = "contentBinaryVersion")]
    pub version: String,
}

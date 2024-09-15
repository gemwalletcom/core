use primitives::Platform;
use serde::{Deserialize, Serialize};
use std::error::Error;
use storage::{database::DatabaseClient, models::Version};

pub struct Client {
    database: DatabaseClient,
}

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

impl Client {
    pub fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub async fn update_ios_version(&mut self) -> Result<Version, Box<dyn Error + Send + Sync>> {
        let ios_version = self.get_app_store_version().await?;
        let version = Version {
            id: 0,
            platform: Platform::IOS.as_str().to_string(),
            production: ios_version.clone(),
            beta: ios_version.clone(),
            alpha: ios_version.clone(),
        };
        let _ = self.database.set_version(version.clone())?;
        Ok(version)
    }

    pub async fn update_apk_version(&mut self) -> Result<Version, Box<dyn Error + Send + Sync>> {
        let version = self.get_github_apk_version().await?;
        let version = Version {
            id: 0,
            platform: Platform::Android.as_str().to_string(),
            production: version.clone(),
            beta: version.clone(),
            alpha: version.clone(),
        };
        let _ = self.database.set_version(version.clone())?;
        Ok(version)
    }

    pub async fn get_app_store_version(&mut self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = "https://itunes.apple.com/lookup?bundleId=com.gemwallet.ios";
        let response = reqwest::get(url).await?.json::<ITunesLookupResponse>().await?;
        let result = response.results.first().expect("expect result");
        Ok(result.version.to_string())
    }

    pub async fn get_github_apk_version(&mut self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = "https://api.github.com/repos/gemwalletcom/gem-android/releases";
        let client = reqwest::Client::builder().user_agent("").build()?;
        let response = client.get(url).send().await?.json::<Vec<GitHubRepository>>().await?;
        let results = response.into_iter().filter(|x| !x.draft && !x.prerelease).collect::<Vec<_>>();
        let result = results.first().expect("expect github repository");
        Ok(result.name.clone())
    }
}

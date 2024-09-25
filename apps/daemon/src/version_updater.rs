use primitives::{config::Release, PlatformStore};
use serde::{Deserialize, Serialize};
use std::error::Error;
use storage::database::DatabaseClient;

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

    pub async fn update_ios_version(&mut self) -> Result<Release, Box<dyn Error + Send + Sync>> {
        let version = self.get_app_store_version().await?;
        let release = Release {
            store: PlatformStore::AppStore,
            version: version.clone(),
            upgrade_required: false,
        };
        self.set_release(release)
    }

    pub async fn update_apk_version(&mut self) -> Result<Release, Box<dyn Error + Send + Sync>> {
        let version = self.get_github_apk_version().await?;
        let release = Release {
            store: PlatformStore::ApkUniversal,
            version: version.clone(),
            upgrade_required: false,
        };
        self.set_release(release)
    }

    fn set_release(&mut self, release: Release) -> Result<Release, Box<dyn Error + Send + Sync>> {
        let releases = vec![storage::models::Release::from_primitive(release.clone()).clone()];
        let _ = self.database.update_releases(releases)?;
        Ok(release)
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

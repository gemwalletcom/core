use primitives::{config::Release, PlatformStore};
use std::error::Error;
use storage::database::DatabaseClient;

use super::model::{GitHubRepository, ITunesLookupResponse, SamsungStoreDetail};

pub struct VersionClient {
    database: DatabaseClient,
}

impl VersionClient {
    pub fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub async fn update_ios_version(&mut self) -> Result<Release, Box<dyn Error + Send + Sync>> {
        let version = self.get_app_store_version().await?;
        self.set_release(Release::new(PlatformStore::AppStore, version.clone(), false))
    }

    pub async fn update_apk_version(&mut self) -> Result<Release, Box<dyn Error + Send + Sync>> {
        let version = self.get_github_apk_version().await?;
        self.set_release(Release::new(PlatformStore::ApkUniversal, version.clone(), false))
    }

    pub async fn update_samsung_store_version(&mut self) -> Result<Release, Box<dyn Error + Send + Sync>> {
        let url = "https://galaxystore.samsung.com/api/detail/com.gemwallet.android";
        let response = reqwest::get(url).await?.json::<SamsungStoreDetail>().await?;
        Ok(Release::new(PlatformStore::SamsungStore, response.details.version.clone(), false))
    }

    fn set_release(&mut self, release: Release) -> Result<Release, Box<dyn Error + Send + Sync>> {
        let releases = storage::models::Release::from_primitive(release.clone()).clone();
        let _ = self.database.update_release(releases)?;
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

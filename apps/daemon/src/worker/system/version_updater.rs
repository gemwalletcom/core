use primitives::{PlatformStore, config::Release};
use std::error::Error;
use storage::{Database, ReleasesRepository};

use super::model::{GitHubRepository, ITunesLookupResponse, SamsungStoreDetail};

pub struct VersionUpdater {
    database: Database,
}

impl VersionUpdater {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn stores() -> &'static [PlatformStore] {
        &[PlatformStore::AppStore, PlatformStore::ApkUniversal, PlatformStore::SamsungStore]
    }

    pub async fn update_store(&self, store: PlatformStore) -> Result<String, Box<dyn Error + Send + Sync>> {
        let version = self.get_store_version(store).await?;
        let current = self.get_current_version(store)?;

        if current.as_ref() != Some(&version) {
            self.set_release(Release::new(store, version.clone(), false))?;
        }

        Ok(version)
    }

    fn get_current_version(&self, store: PlatformStore) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let releases = self.database.releases()?.get_releases()?;
        let version = releases.into_iter().find(|r| r.platform_store.0 == store).map(|r| r.version);
        Ok(version)
    }

    async fn get_store_version(&self, store: PlatformStore) -> Result<String, Box<dyn Error + Send + Sync>> {
        match store {
            PlatformStore::AppStore => self.get_app_store_version().await,
            PlatformStore::ApkUniversal => self.get_github_version().await,
            PlatformStore::SamsungStore => self.get_samsung_version().await,
            _ => Err(format!("unsupported store: {:?}", store).into()),
        }
    }

    fn set_release(&self, release: Release) -> Result<(), Box<dyn Error + Send + Sync>> {
        let row = storage::models::ReleaseRow::from_primitive(release);
        self.database.releases()?.update_release(row)?;
        Ok(())
    }

    async fn get_app_store_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = "https://itunes.apple.com/lookup?bundleId=com.gemwallet.ios";
        let response = reqwest::get(url).await?.json::<ITunesLookupResponse>().await?;
        response.results.first().map(|r| r.version.clone()).ok_or_else(|| "no results".into())
    }

    async fn get_github_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = "https://api.github.com/repos/gemwalletcom/gem-android/releases";
        let response = reqwest::Client::builder()
            .user_agent("gem-daemon")
            .build()?
            .get(url)
            .send()
            .await?
            .json::<Vec<GitHubRepository>>()
            .await?;
        response
            .into_iter()
            .find(|x| !x.draft && !x.prerelease && x.assets.iter().any(|a| a.name.contains("gem_wallet_universal_")))
            .map(|r| r.name)
            .ok_or_else(|| "no releases".into())
    }

    async fn get_samsung_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = "https://galaxystore.samsung.com/api/detail/com.gemwallet.android";
        let response = reqwest::get(url).await?.json::<SamsungStoreDetail>().await?;
        match response.details {
            Some(details) => Ok(details.version),
            None => Err(response.error_message.unwrap_or_else(|| "no version found".to_string()).into()),
        }
    }
}

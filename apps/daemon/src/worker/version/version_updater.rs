use gem_tracing::info_with_fields;
use primitives::{PlatformStore, config::Release};
use std::error::Error;
use storage::{Database, ReleasesRepository};

use super::model::{GitHubRepository, ITunesLookupResponse, SamsungStoreDetail};

pub struct VersionClient {
    database: Database,
}

impl VersionClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn update_store_versions(&self) -> Result<Vec<(String, String)>, Box<dyn Error + Send + Sync>> {
        let platforms = [PlatformStore::AppStore, PlatformStore::ApkUniversal, PlatformStore::SamsungStore];
        let mut updates = Vec::new();

        for platform in platforms {
            let version = match platform {
                PlatformStore::AppStore => self.update_app_store_version().await?,
                PlatformStore::ApkUniversal => self.update_apk_version().await?,
                PlatformStore::SamsungStore => self.update_samsung_store_version().await?,
                _ => continue,
            };

            info_with_fields!("update_store_version", platform = platform.as_ref(), version = version.as_str());
            updates.push((platform.as_ref().to_string(), version));
        }
        Ok(updates)
    }

    async fn update_app_store_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let version = self.get_app_store_version().await?;
        self.set_release(Release::new(PlatformStore::AppStore, version.clone(), false))?;
        Ok(version)
    }

    async fn update_apk_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let version = self.get_github_apk_version().await?;
        self.set_release(Release::new(PlatformStore::ApkUniversal, version.clone(), false))?;
        Ok(version)
    }

    async fn update_samsung_store_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = "https://galaxystore.samsung.com/api/detail/com.gemwallet.android";
        let response = reqwest::get(url).await?.json::<SamsungStoreDetail>().await?;
        let version = response.details.version.clone();
        self.set_release(Release::new(PlatformStore::SamsungStore, version.clone(), false))?;
        Ok(version)
    }

    fn set_release(&self, release: Release) -> Result<Release, Box<dyn Error + Send + Sync>> {
        let releases = storage::models::ReleaseRow::from_primitive(release.clone()).clone();
        let _ = self.database.releases()?.update_release(releases)?;
        Ok(release)
    }

    async fn get_app_store_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = "https://itunes.apple.com/lookup?bundleId=com.gemwallet.ios";
        let response = reqwest::get(url).await?.json::<ITunesLookupResponse>().await?;
        let version = response
            .results
            .first()
            .map(|result| result.version.to_string())
            .ok_or_else(|| "app store lookup returned no results".to_string())?;
        Ok(version)
    }

    async fn get_github_apk_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = "https://api.github.com/repos/gemwalletcom/gem-android/releases";
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?.json::<Vec<GitHubRepository>>().await?;
        let results = response
            .into_iter()
            .filter(|x| !x.draft && !x.prerelease && x.assets.clone().into_iter().any(|x| x.name.contains("gem_wallet_universal_")))
            .collect::<Vec<_>>();
        let version = results
            .first()
            .map(|result| result.name.clone())
            .ok_or_else(|| "github releases list is empty".to_string())?;
        Ok(version)
    }
}

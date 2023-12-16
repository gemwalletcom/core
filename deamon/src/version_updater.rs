use primitives::platform::Platform;
use std::error::Error;
use storage::{database::DatabaseClient, models::Version};

pub struct Client {
    database: DatabaseClient,
}

impl Client {
    pub fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub async fn update_ios_version(&mut self) -> Result<Version, Box<dyn Error>> {
        let ios_version = self.get_app_store_version().await.unwrap();
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

    pub async fn get_app_store_version(&self) -> Result<String, Box<dyn Error>> {
        let url = "https://itunes.apple.com/lookup?bundleId=com.gemwallet.ios";
        let resp = reqwest::get(url).await?;
        let json = resp.json::<serde_json::Value>().await?;
        let version = json["results"][0]["version"].as_str().unwrap_or_default();
        Ok(version.to_string())
    }

    // pub async fn get_google_play_version(&self) -> Result<String, Box<dyn Error>> {
    //     let url = "https://play.google.com/store/apps/details?id=com.gemwallet.android";
    //     let resp = reqwest::get(url).await?;
    //     let body = resp.text().await?;
    //     let version = body.split("Current Version").collect::<Vec<&str>>()[1]
    //         .split("<div class=\"BgcNfc\">")[1]
    //         .split("</div>")[0]
    //         .trim();
    //     Ok(version.to_string())
    // }
}

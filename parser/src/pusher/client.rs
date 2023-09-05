use super::model::{Notifications, Response};
pub struct PusherClient {
    url: String,
    client: reqwest::Client,
}

impl PusherClient {
    pub fn new(
        url: String,
    ) -> Self {
        let client = reqwest::Client::new();
        Self {
            url,
            client,
        }
    }

    pub async fn push(&self, notifications: Notifications) -> Result<usize, reqwest::Error> {
        let url = format!("{}/api/push", self.url);

        let _ = self.client
            .post(&url)
            .json(&notifications)
            .send()
            .await?
            .json::<Response>()
            .await?;
    
        Ok(notifications.notifications.len())
    }
}
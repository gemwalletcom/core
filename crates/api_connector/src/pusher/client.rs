use super::model::{Notification, Notifications, Response};
pub struct PusherClient {
    url: String,
    client: reqwest::Client,
}

impl PusherClient {
    pub fn new(url: String) -> Self {
        let client = reqwest::Client::new();
        Self { url, client }
    }

    pub async fn push_notifications(&self, notifications: Vec<Notification>) -> Result<Response, reqwest::Error> {
        let url = format!("{}/api/push", self.url);
        let notifications = Notifications { notifications };

        let response = self.client.post(&url).json(&notifications).send().await?.json::<Response>().await?;

        Ok(response)
    }

    pub async fn push(&self, notification: Notification) -> Result<Response, reqwest::Error> {
        self.push_notifications(vec![notification]).await
    }
}

use super::model::{Notifications, Response, Notification};
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

    pub async fn push(&self, notification: Notification) -> Result<Response, reqwest::Error> {
        let url = format!("{}/api/push", self.url);

        let notifications = Notifications {
            notifications: vec![notification]
        };

        let response = self.client
            .post(&url)
            .json(&notifications)
            .send()
            .await?
            .json::<Response>()
            .await?;
    
        Ok(response)
    }
}
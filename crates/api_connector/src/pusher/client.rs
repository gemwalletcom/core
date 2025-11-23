use super::model::{PushResult, Response};
use primitives::{GorushNotification, GorushNotifications};
use reqwest::Client;

#[derive(Clone, Debug)]
pub struct PusherClient {
    url: String,
    client: Client,
    topic: String,
}

impl PusherClient {
    pub fn new(url: String, topic: String) -> Self {
        let client = Client::new();
        Self { url, client, topic }
    }

    pub async fn push_notifications(&self, notifications: Vec<GorushNotification>) -> Result<PushResult, reqwest::Error> {
        let url = format!("{}/api/push", self.url);
        let notifications: Vec<GorushNotification> = notifications
            .into_iter()
            .filter(|n| !n.tokens.is_empty() && n.tokens.iter().all(|t| !t.is_empty()))
            .map(|x| x.clone().with_topic(self.get_topic(x.platform)))
            .collect();
        let payload = GorushNotifications {
            notifications: notifications.clone(),
        };
        let response = self.client.post(&url).json(&payload).send().await?.json::<Response>().await?;
        Ok(PushResult { response, notifications })
    }

    //Remove in the future
    fn get_topic(&self, platform: i32) -> Option<String> {
        match platform {
            1 => Some(self.topic.clone()), // ios
            2 => None,
            _ => None,
        }
    }
}

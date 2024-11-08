use super::model::{Notification, Notifications, Response};
use primitives::{Platform, PushNotification};

#[derive(Clone, Debug)]
pub struct PusherClient {
    url: String,
    client: reqwest::Client,
    topic: String,
}

impl PusherClient {
    pub fn new(url: String, topic: String) -> Self {
        let client = reqwest::Client::new();
        Self { url, client, topic }
    }

    pub async fn push_notifications(&self, notifications: Vec<Notification>) -> Result<Response, reqwest::Error> {
        let url = format!("{}/api/push", self.url);
        let notifications = notifications.into_iter().map(|x| x.clone().with_topic(self.get_topic(x.platform))).collect();
        let notifications = Notifications { notifications };

        let response = self.client.post(&url).json(&notifications).send().await?.json::<Response>().await?;

        Ok(response)
    }

    pub fn new_notification(&self, token: &str, platform: Platform, title: &str, body: &str, data: PushNotification) -> Notification {
        Notification::new(vec![token.to_owned()], platform.as_i32(), title.to_owned(), body.to_owned(), data)
    }

    //Remove in the future
    fn get_topic(&self, platform: i32) -> Option<String> {
        match platform {
            1 => Some(self.topic.clone()), // ios
            2 => None,
            _ => None,
        }
    }

    pub async fn push(&self, notification: Notification) -> Result<Response, reqwest::Error> {
        self.push_notifications(vec![notification]).await
    }
}

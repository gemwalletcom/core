use primitives::{FailedNotification, GorushNotification, PushErrorLog};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub counts: i32,
    pub logs: Vec<PushErrorLog>,
    pub success: String,
}

pub struct PushResult {
    pub response: Response,
    pub notifications: Vec<GorushNotification>,
}

impl PushResult {
    pub fn failures(self) -> Vec<FailedNotification> {
        let token_to_notification: HashMap<String, GorushNotification> = self
            .notifications
            .into_iter()
            .flat_map(|n| n.tokens.clone().into_iter().map(move |t| (t, n.clone())).collect::<Vec<_>>())
            .collect();

        self.response
            .logs
            .into_iter()
            .filter(|log| !log.error.is_empty())
            .filter_map(|error| {
                token_to_notification.get(&error.token).map(|notification| FailedNotification {
                    notification: notification.clone(),
                    error,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub title: String,
    pub message: Option<String>,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::GorushNotification;

    #[test]
    fn failures_matches_tokens() {
        let response = Response {
            counts: 2,
            success: "ok".to_string(),
            logs: vec![
                PushErrorLog {
                    token: "token1".to_string(),
                    error: "BadDeviceToken".to_string(),
                },
                PushErrorLog {
                    token: "token2".to_string(),
                    error: "Requested entity was not found.".to_string(),
                },
            ],
        };

        let result = PushResult {
            response,
            notifications: vec![
                GorushNotification::mock_with("token1", "device1"),
                GorushNotification::mock_with("token2", "device2"),
            ],
        };

        let failures = result.failures();
        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0].notification.device_id, "device1");
        assert_eq!(failures[0].error.error, "BadDeviceToken");
        assert_eq!(failures[1].notification.device_id, "device2");
        assert_eq!(failures[1].error.error, "Requested entity was not found.");
    }

    #[test]
    fn failures_filters_invalid() {
        let response = Response {
            counts: 3,
            success: "ok".to_string(),
            logs: vec![
                PushErrorLog {
                    token: "token1".to_string(),
                    error: "".to_string(),
                },
                PushErrorLog {
                    token: "unmatched".to_string(),
                    error: "BadDeviceToken".to_string(),
                },
                PushErrorLog {
                    token: "token2".to_string(),
                    error: "Error".to_string(),
                },
            ],
        };

        let result = PushResult {
            response,
            notifications: vec![
                GorushNotification::mock_with("token1", "device1"),
                GorushNotification::mock_with("token2", "device2"),
            ],
        };

        let failures = result.failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].error.token, "token2");
    }
}

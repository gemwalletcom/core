use crate::{Device, PushNotification};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GorushNotifications {
    pub notifications: Vec<GorushNotification>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GorushNotification {
    pub tokens: Vec<String>,
    pub platform: i32,
    pub title: String,
    pub message: String,
    pub topic: Option<String>,
    pub data: PushNotification,
    pub device_id: String,
}

impl GorushNotification {
    pub fn from_device(device: Device, title: String, message: String, data: PushNotification) -> Self {
        Self {
            tokens: vec![device.token.clone()],
            platform: device.platform.as_i32(),
            title,
            message,
            topic: None,
            data,
            device_id: device.id,
        }
    }

    pub fn with_topic(mut self, topic: Option<String>) -> Self {
        self.topic = topic;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedNotification {
    pub notification: GorushNotification,
    pub error: PushErrorLog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushErrorLog {
    pub token: String,
    pub error: String,
}

impl PushErrorLog {
    pub fn new(token: String, error: String) -> Self {
        Self { token, error }
    }

    pub fn is_device_invalid(&self) -> bool {
        const ERROR_PATTERNS: &[&str] = &[
            "notregistered",
            "unregistered",
            "invalidregistration",
            "baddevicetoken",
            "devicetokennotfortopic",
            "mismatchsenderid",
            "requested entity was not found",
        ];

        let error_lower = self.error.to_lowercase();
        ERROR_PATTERNS.iter().any(|pattern| error_lower.contains(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_device_invalid() {
        assert!(PushErrorLog::new("test".to_string(), "Unregistered".to_string()).is_device_invalid());
        assert!(PushErrorLog::new("test".to_string(), "InvalidRegistration".to_string()).is_device_invalid());
        assert!(PushErrorLog::new("test".to_string(), "BadDeviceToken".to_string()).is_device_invalid());
        assert!(PushErrorLog::new("test".to_string(), "Devicetokennotfortopic".to_string()).is_device_invalid());
        assert!(PushErrorLog::new("test".to_string(), "Mismatchsenderid".to_string()).is_device_invalid());

        assert!(!PushErrorLog::new("".to_string(), "Good".to_string()).is_device_invalid());
    }
}

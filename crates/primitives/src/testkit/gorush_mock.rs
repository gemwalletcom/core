use crate::{GorushNotification, Platform, PushNotification, PushNotificationTypes};

impl GorushNotification {
    pub fn mock() -> Self {
        Self {
            tokens: vec!["test-token".to_string()],
            platform: Platform::Android.as_i32(),
            title: "Test".to_string(),
            message: "Test".to_string(),
            topic: None,
            data: PushNotification {
                data: None,
                notification_type: PushNotificationTypes::Transaction,
            },
            device_id: "test-device-id".to_string(),
        }
    }

    pub fn mock_with(token: &str, device_id: &str) -> Self {
        Self {
            tokens: vec![token.to_string()],
            device_id: device_id.to_string(),
            ..Self::mock()
        }
    }
}

use crate::{Device, Platform};

impl Device {
    pub fn mock() -> Self {
        Self {
            id: "test-device-id".to_string(),
            platform: Platform::IOS,
            os: Some("iOS 17.0".to_string()),
            model: Some("iPhone 15".to_string()),
            platform_store: None,
            token: "test-token-123".to_string(),
            locale: "en".to_string(),
            version: "1.0.0".to_string(),
            currency: "USD".to_string(),
            is_push_enabled: true,
            is_price_alerts_enabled: Some(true),
            subscriptions_version: 1,
        }
    }

    pub fn mock_with(is_push_enabled: bool, token: String, is_price_alerts_enabled: Option<bool>) -> Self {
        Self {
            is_push_enabled,
            token,
            is_price_alerts_enabled,
            ..Self::mock()
        }
    }
}

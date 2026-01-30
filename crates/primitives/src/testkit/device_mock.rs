use crate::{Device, Platform, PlatformStore};

impl Device {
    pub fn mock() -> Self {
        Self {
            id: "test-device-id".to_string(),
            platform: Platform::IOS,
            platform_store: PlatformStore::AppStore,
            os: "iOS 17.0".to_string(),
            model: "iPhone 15".to_string(),
            token: "test-token-123".to_string(),
            locale: "en".to_string(),
            version: "1.0.0".to_string(),
            currency: "USD".to_string(),
            is_push_enabled: true,
            is_price_alerts_enabled: Some(true),
            subscriptions_version: 1,
            public_key: None,
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

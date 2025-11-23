use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{PlatformStore, platform::Platform};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub platform: Platform,
    pub os: Option<String>,
    pub model: Option<String>,
    pub platform_store: Option<PlatformStore>,
    pub token: String,
    pub locale: String,
    pub version: String,
    pub currency: String,
    pub is_push_enabled: bool,
    pub is_price_alerts_enabled: Option<bool>,
    pub subscriptions_version: i32,
}

impl Device {
    pub fn can_receive_push_notification(&self) -> bool {
        self.is_push_enabled && !self.token.is_empty()
    }

    pub fn can_receive_price_alerts(&self) -> bool {
        self.can_receive_push_notification() && self.is_price_alerts_enabled == Some(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_receive_push_notification() {
        assert!(Device::mock().can_receive_push_notification());
        assert!(!Device::mock_with(false, "token".to_string(), Some(true)).can_receive_push_notification());
        assert!(!Device::mock_with(true, "".to_string(), Some(true)).can_receive_push_notification());
    }

    #[test]
    fn can_receive_price_alerts() {
        assert!(Device::mock().can_receive_price_alerts());
        assert!(!Device::mock_with(true, "token".to_string(), Some(false)).can_receive_price_alerts());
        assert!(!Device::mock_with(true, "token".to_string(), None).can_receive_price_alerts());
        assert!(!Device::mock_with(false, "token".to_string(), Some(true)).can_receive_price_alerts());
        assert!(!Device::mock_with(true, "".to_string(), Some(true)).can_receive_price_alerts());
    }
}

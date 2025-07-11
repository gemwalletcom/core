use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::{Asset, AssetId, Device, Price, DEFAULT_FIAT_CURRENCY};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlert {
    pub asset_id: AssetId,
    #[serde(default = "default_currency")]
    pub currency: String,
    pub price: Option<f64>,
    pub price_percent_change: Option<f64>,
    pub price_direction: Option<PriceAlertDirection>,
    pub last_notified_at: Option<DateTime<Utc>>,
}

fn default_currency() -> String {
    DEFAULT_FIAT_CURRENCY.to_string()
}

impl PriceAlert {
    pub fn id(&self) -> String {
        if self.price.is_none() && self.price_percent_change.is_none() && self.price_direction.is_none() {
            return self.asset_id.to_string();
        }
        let parts: Vec<String> = vec![
            Some(self.asset_id.to_string()),
            Some(self.currency.clone()),
            self.price.map(|p| p.to_string()),
            self.price_percent_change.map(|p| p.to_string()),
            self.price_direction.clone().map(|d| d.as_ref().to_string()),
        ]
        .into_iter()
        .filter_map(|x| x.map(|s| s.to_string()))
        .collect();
        parts.join("_")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlertData {
    pub asset: Asset,
    pub price: Option<Price>,
    pub price_alert: PriceAlert,
}

#[derive(Clone, Debug, Serialize, Deserialize, AsRefStr, EnumString, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PriceAlertDirection {
    Up,
    Down,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PriceAlertType {
    PriceChangesUp,
    PriceChangesDown,
    PriceUp,
    PriceDown,
    PricePercentChangeUp,
    PricePercentChangeDown,
    AllTimeHigh,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub enum PriceAlertNotificationType {
    Auto,
    Price,
    PricePercentChange,
}

pub type PriceAlerts = Vec<PriceAlert>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct DevicePriceAlert {
    pub device: Device,
    pub price_alert: PriceAlert,
}

#[cfg(test)]
mod tests {
    use crate::Chain;

    use super::*;

    #[test]
    fn test_price_alert_id_with_all_fields() {
        let price_alert = PriceAlert {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            currency: "USD".to_string(),
            price: Some(100.0),
            price_percent_change: Some(5.0),
            price_direction: Some(PriceAlertDirection::Up),
            last_notified_at: None,
        };
        assert_eq!(price_alert.id(), "ethereum_USD_100_5_up");

        let price_alert = PriceAlert {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            currency: "USD".to_string(),
            price: Some(1.12344),
            price_percent_change: Some(10_000.10),
            price_direction: Some(PriceAlertDirection::Up),
            last_notified_at: None,
        };
        assert_eq!(price_alert.id(), "ethereum_USD_1.12344_10000.1_up");
    }

    #[test]
    fn test_price_alert_id_with_missing_optional_fields() {
        let price_alert = PriceAlert {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            currency: "USD".to_string(),
            price: None,
            price_percent_change: None,
            price_direction: None,
            last_notified_at: None,
        };
        assert_eq!(price_alert.id(), "ethereum");
    }

    #[test]
    fn test_price_alert_id_with_some_optional_fields() {
        let price_alert = PriceAlert {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            currency: "USD".to_string(),
            price: Some(100.0),
            price_percent_change: None,
            price_direction: Some(PriceAlertDirection::Down),
            last_notified_at: None,
        };
        assert_eq!(price_alert.id(), "ethereum_USD_100_down");
    }
}

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatProvider {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
    #[serde(skip_serializing)]
    #[typeshare(skip)]
    pub priority: Option<i32>,
    #[serde(skip_serializing)]
    #[typeshare(skip)]
    pub threshold_bps: Option<i32>,
    #[serde(skip_serializing)]
    #[typeshare(skip)]
    pub enabled: bool,
    #[serde(skip_serializing)]
    #[typeshare(skip)]
    pub buy_enabled: bool,
    #[serde(skip_serializing)]
    #[typeshare(skip)]
    pub sell_enabled: bool,
}

impl FiatProvider {
    pub fn is_buy_enabled(&self) -> bool {
        self.enabled && self.buy_enabled
    }

    pub fn is_sell_enabled(&self) -> bool {
        self.enabled && self.sell_enabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum FiatProviderName {
    Mercuryo,
    Transak,
    MoonPay,
    Ramp,
    Banxa,
    Paybis,
}

impl FiatProviderName {
    pub fn id(&self) -> String {
        self.as_ref().to_string()
    }

    pub fn name(&self) -> &'static str {
        match self {
            FiatProviderName::Mercuryo => "Mercuryo",
            FiatProviderName::Transak => "Transak",
            FiatProviderName::MoonPay => "MoonPay",
            FiatProviderName::Ramp => "Ramp",
            FiatProviderName::Banxa => "Banxa",
            FiatProviderName::Paybis => "Paybis",
        }
    }
    pub fn as_fiat_provider(&self) -> FiatProvider {
        FiatProvider {
            id: self.id(),
            name: self.name().to_owned(),
            image_url: Some("".to_string()),
            priority: None,
            threshold_bps: None,
            enabled: true,
            buy_enabled: true,
            sell_enabled: true,
        }
    }

    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone)]
pub struct FiatProviderCountry {
    pub provider: String,
    pub alpha2: String,
    pub is_allowed: bool,
}

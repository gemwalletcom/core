use crate::{PaymentType, PrioritizedProvider};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatProvider {
    #[typeshare(serialized_as = "String")]
    pub id: FiatProviderName,
    pub name: String,
    pub image_url: Option<String>,
    #[serde(skip)]
    #[typeshare(skip)]
    pub priority: Option<i32>,
    #[serde(skip)]
    #[typeshare(skip)]
    pub threshold_bps: Option<i32>,
    #[serde(skip)]
    #[typeshare(skip)]
    pub enabled: bool,
    #[serde(skip)]
    #[typeshare(skip)]
    pub buy_enabled: bool,
    #[serde(skip)]
    #[typeshare(skip)]
    pub sell_enabled: bool,
    pub payment_methods: Vec<PaymentType>,
}

impl FiatProvider {
    pub fn is_buy_enabled(&self) -> bool {
        self.enabled && self.buy_enabled
    }

    pub fn is_sell_enabled(&self) -> bool {
        self.enabled && self.sell_enabled
    }
}

impl PrioritizedProvider for FiatProvider {
    fn provider_id(&self) -> &str {
        self.id.as_ref()
    }

    fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    fn threshold_bps(&self) -> i32 {
        self.threshold_bps.unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FiatProviderName {
    Mercuryo,
    Transak,
    MoonPay,
    Banxa,
    Paybis,
    Flashnet,
}

impl FiatProviderName {
    pub fn id(&self) -> &str {
        self.as_ref()
    }

    pub fn name(&self) -> &'static str {
        match self {
            FiatProviderName::Mercuryo => "Mercuryo",
            FiatProviderName::Transak => "Transak",
            FiatProviderName::MoonPay => "MoonPay",
            FiatProviderName::Banxa => "Banxa",
            FiatProviderName::Paybis => "Paybis",
            FiatProviderName::Flashnet => "CashApp",
        }
    }
    pub fn as_fiat_provider(&self) -> FiatProvider {
        let enabled = *self != Self::Banxa;
        FiatProvider {
            id: *self,
            name: self.name().to_owned(),
            image_url: None,
            priority: None,
            threshold_bps: None,
            enabled,
            buy_enabled: enabled,
            sell_enabled: enabled,
            payment_methods: vec![],
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Mercuryo, Self::Transak, Self::MoonPay, Self::Paybis, Self::Flashnet]
    }
}

#[derive(Debug, Clone)]
pub struct FiatProviderCountry {
    pub provider: String,
    pub alpha2: String,
    pub is_allowed: bool,
}

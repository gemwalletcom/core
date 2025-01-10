use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct FiatProvider {
    pub id: String,
    pub name: String,
    pub image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum FiatProviderName {
    Mercuryo,
    Transak,
    MoonPay,
    Ramp,
    Banxa,
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
        }
    }
    pub fn as_fiat_provider(&self) -> FiatProvider {
        FiatProvider {
            id: self.id(),
            name: self.name().to_owned(),
            image_url: "".to_string(),
        }
    }

    pub fn all() -> Vec<FiatProviderName> {
        FiatProviderName::iter().collect::<Vec<_>>()
    }
}

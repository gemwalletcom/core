use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable")]
#[serde(rename_all = "camelCase")]
pub struct FiatProvider {
    pub name: String,
    pub image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter)]
pub enum FiatProviderName {
    Mercuryo,
    Transak,
    MoonPay,
    Ramp,
    Banxa,
    Kado,
}

impl FiatProviderName {
    pub fn as_str(&self) -> &'static str {
        match self {
            FiatProviderName::Mercuryo => "Mercuryo",
            FiatProviderName::Transak => "Transak",
            FiatProviderName::MoonPay => "MoonPay",
            FiatProviderName::Ramp => "Ramp",
            FiatProviderName::Banxa => "Banxa",
            FiatProviderName::Kado => "Kado",
        }
    }

    pub fn id(&self) -> String {
        self.as_str().to_lowercase()
    }

    pub fn as_fiat_provider(&self) -> FiatProvider {
        FiatProvider {
            name: self.as_str().to_string(),
            image_url: "".to_string(),
        }
    }

    pub fn all() -> Vec<FiatProviderName> {
        FiatProviderName::iter().collect::<Vec<_>>()
    }
}

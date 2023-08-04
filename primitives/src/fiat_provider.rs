
use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable")]
#[serde(rename_all = "camelCase")]
pub struct FiatProvider {
    pub name: String,
    pub image_url: String,
}

pub enum FiatProviderName {
    Mercuryo,
    Transak,
    MoonPay,
    Ramp,
}

impl FiatProviderName {
    pub fn as_str(&self) -> &'static str {
        match self {
            FiatProviderName::Mercuryo => "Mercuryo",
            FiatProviderName::Transak => "Transak",
            FiatProviderName::MoonPay => "MoonPay",
            FiatProviderName::Ramp => "Ramp",
        }
    }

    pub fn as_fiat_provider(&self) -> FiatProvider {
        return FiatProvider { 
            name: self.as_str().to_string(),
            image_url: "".to_string(),
        }
    }
}
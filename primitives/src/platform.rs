use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift="Codable")]
#[serde(rename_all = "lowercase")]
pub enum Platform  {
    IOS,
    Android,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::IOS => "ios",
            Platform::Android => "android",
        }
    }

    pub fn from_str(s: &str) -> Option<Platform> {
        match s {
            "ios" => Some(Platform::IOS),
            "android" => Some(Platform::Android),
            _ => None,
        }
    }
}
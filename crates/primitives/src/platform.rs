use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Equatable")]
#[serde(rename_all = "lowercase")]
pub enum Platform {
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

    pub fn as_i32(&self) -> i32 {
        match self {
            Platform::IOS => 1,
            Platform::Android => 2,
        }
    }

    pub fn new(s: &str) -> Option<Platform> {
        match s {
            "ios" => Some(Platform::IOS),
            "android" => Some(Platform::Android),
            _ => None,
        }
    }
}

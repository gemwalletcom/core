use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, AsRefStr, Display)]
#[strum(serialize_all = "camelCase")]
pub enum IpUsageType {
    DataCenter,
    Hosting,
    Isp,
    Mobile,
    Business,
    Education,
    Government,
    #[default]
    Unknown,
}

impl IpUsageType {
    pub fn is_datacenter(&self) -> bool {
        matches!(self, Self::DataCenter | Self::Hosting)
    }
}

impl FromStr for IpUsageType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        let result = match lower.as_str() {
            "datacenter" | "data_center" => Self::DataCenter,
            "hosting" => Self::Hosting,
            "isp" => Self::Isp,
            "mobile" => Self::Mobile,
            "business" => Self::Business,
            "education" => Self::Education,
            "government" => Self::Government,
            _ if lower.contains("data center") || lower.contains("web hosting") || lower.contains("transit") => Self::DataCenter,
            _ if lower.contains("hosting") => Self::Hosting,
            _ if lower.contains("mobile") => Self::Mobile,
            _ if lower.contains("isp") || lower.contains("fixed line") => Self::Isp,
            _ if lower.contains("commercial") || lower.contains("business") => Self::Business,
            _ if lower.contains("university") || lower.contains("college") || lower.contains("school") || lower.contains("education") => Self::Education,
            _ if lower.contains("government") || lower.contains("military") => Self::Government,
            _ => Self::Unknown,
        };
        Ok(result)
    }
}

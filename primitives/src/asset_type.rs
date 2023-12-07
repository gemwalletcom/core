use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetType {
    NATIVE,
    ERC20,
    BEP2,
    BEP20,
    SPL,
    ARBITRUM,
    TRC20,
}

impl AssetType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "NATIVE" => Some(Self::NATIVE),
            "ERC20" => Some(Self::ERC20),
            "BEP2" => Some(Self::BEP2),
            "BEP20" => Some(Self::BEP20),
            "SPL" => Some(Self::SPL),
            "ARBITRUM" => Some(Self::ARBITRUM),
            "TRC20" => Some(Self::TRC20),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::NATIVE => "NATIVE",
            Self::ERC20 => "ERC20",
            Self::BEP2 => "BEP2",
            Self::BEP20 => "BEP20",
            Self::SPL => "SPL",
            Self::ARBITRUM => "ARBITRUM",
            Self::TRC20 => "TRC20",
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetSubtype {
    NATIVE,
    TOKEN,
}

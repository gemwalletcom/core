use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
pub struct AssetScore {
    pub rank: i32,
    #[typeshare(skip)]
    #[serde(rename = "type")]
    pub rank_type: AssetRank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum AssetScoreType {
    Verified,
    Unverified,
    Suspicious,
}

impl AssetScore {
    pub fn new(rank: i32) -> Self {
        let rank_type = AssetRank::from_rank(rank);
        Self { rank, rank_type }
    }

    pub fn rank_type(&self) -> AssetRank {
        AssetRank::from_rank(self.rank)
    }
}

impl Default for AssetScore {
    fn default() -> Self {
        Self::new(15)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum AssetRank {
    High,
    Medium,
    Low,
    Trivial,
    Inactive,
    Abandoned,
    Suspended,
    Migrated,
    Deprecated,
    Spam,
    Fraudulent,
    Unknown,
}

impl AssetRank {
    pub fn threshold(&self) -> i32 {
        match self {
            AssetRank::High => 100,
            AssetRank::Medium => 50,
            AssetRank::Low => 25,
            AssetRank::Trivial => 15,
            AssetRank::Unknown => 0,
            AssetRank::Inactive => -2,
            AssetRank::Abandoned => -5,
            AssetRank::Suspended => -8,
            AssetRank::Migrated => -10,
            AssetRank::Deprecated => -12,
            AssetRank::Spam => -15,
            AssetRank::Fraudulent => -20,
        }
    }

    pub fn from_rank(rank: i32) -> Self {
        use strum::IntoEnumIterator;
        AssetRank::iter().find(|variant| rank >= variant.threshold()).unwrap_or(AssetRank::Unknown)
    }
}

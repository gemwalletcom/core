use crate::SwapProvider;
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

#[derive(Debug, Copy, Clone, PartialEq, Eq, AsRefStr, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum CrossChainProvider {
    Thorchain,
    Across,
    Mayan,
    NearIntents,
}

impl CrossChainProvider {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

impl From<CrossChainProvider> for SwapProvider {
    fn from(value: CrossChainProvider) -> Self {
        match value {
            CrossChainProvider::Thorchain => Self::Thorchain,
            CrossChainProvider::Across => Self::Across,
            CrossChainProvider::Mayan => Self::Mayan,
            CrossChainProvider::NearIntents => Self::NearIntents,
        }
    }
}

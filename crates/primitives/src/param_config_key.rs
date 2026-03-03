use crate::CrossChainProvider;
use strum::AsRefStr;

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "camelCase")]
pub enum ParamConfigKey {
    SwapperVaultAddresses(CrossChainProvider),
}

impl ParamConfigKey {
    pub fn all() -> Vec<Self> {
        CrossChainProvider::all().into_iter().map(Self::SwapperVaultAddresses).collect()
    }

    pub fn key(&self) -> String {
        match self {
            Self::SwapperVaultAddresses(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
        }
    }

    pub fn default_value(&self) -> &str {
        match self {
            Self::SwapperVaultAddresses(provider) => match provider {
                CrossChainProvider::NearIntents => "5s",
                CrossChainProvider::Thorchain => "5m",
                CrossChainProvider::Across | CrossChainProvider::Mayan => "1h",
            },
        }
    }
}

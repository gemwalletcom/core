use crate::SwapProvider;
use strum::AsRefStr;

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "camelCase")]
pub enum ParamConfigKey {
    SwapperVaultAddresses(SwapProvider),
}

impl ParamConfigKey {
    pub fn all() -> Vec<Self> {
        SwapProvider::cross_chain_providers().into_iter().map(Self::SwapperVaultAddresses).collect()
    }

    pub fn key(&self) -> String {
        match self {
            Self::SwapperVaultAddresses(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
        }
    }

    pub fn default_value(&self) -> &str {
        match self {
            Self::SwapperVaultAddresses(_) => "5m",
        }
    }
}

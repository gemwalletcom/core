use serde::{Deserialize, Serialize};

use crate::FiatProviderName;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiatProviderId {
    pub provider: FiatProviderName,
    pub symbol: String,
}

impl FiatProviderId {
    pub fn new(provider: impl Into<FiatProviderName>, symbol: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            symbol: symbol.into(),
        }
    }

    pub fn id(&self) -> String {
        format!("{}_{}", self.provider.id(), self.symbol).to_lowercase()
    }
}

use serde::{Deserialize, Serialize};
use std::fmt;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PriceProvider {
    Coingecko,
    Pyth,
    Jupiter,
}

impl PriceProvider {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn primary() -> Self {
        Self::Coingecko
    }

    pub fn id(&self) -> &str {
        self.as_ref()
    }

    pub fn priority(&self) -> i32 {
        match self {
            Self::Coingecko => 0,
            Self::Pyth => 1,
            Self::Jupiter => 2,
        }
    }

    pub fn price_id(&self, provider_price_id: &str) -> String {
        format!("{}_{}", self.as_ref(), provider_price_id)
    }
}

impl fmt::Display for PriceProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

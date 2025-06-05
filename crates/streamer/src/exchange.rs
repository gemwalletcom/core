use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, EnumIter)]
pub enum ExchangeName {
    Transactions,
}

impl ExchangeName {
    pub fn all() -> Vec<ExchangeName> {
        ExchangeName::iter().collect()
    }
}

impl fmt::Display for ExchangeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExchangeName::Transactions => write!(f, "transactions"),
        }
    }
}

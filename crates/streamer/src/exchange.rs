use std::fmt;

#[derive(Debug, Clone)]
pub enum ExchangeName {
    Transactions,
}

impl fmt::Display for ExchangeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExchangeName::Transactions => write!(f, "transactions"),
        }
    }
}

use std::fmt;

use lapin::ExchangeKind;
use strum::{EnumIter, IntoEnumIterator};

use crate::QueueName;

#[derive(Debug, Clone, EnumIter)]
pub enum ExchangeName {
    NewAddresses,
}

impl ExchangeName {
    pub fn all() -> Vec<ExchangeName> {
        ExchangeName::iter().collect()
    }

    pub fn kind(&self) -> ExchangeKind {
        match self {
            ExchangeName::NewAddresses => ExchangeKind::Topic,
        }
    }

    pub fn queues(&self) -> Vec<QueueName> {
        match self {
            ExchangeName::NewAddresses => vec![
                QueueName::FetchTokenAssociations,
                QueueName::FetchCoinAssociations,
                QueueName::FetchAddressTransactions,
                QueueName::FetchNftAssociations,
            ],
        }
    }
}

impl fmt::Display for ExchangeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExchangeName::NewAddresses => write!(f, "new_addresses"),
        }
    }
}

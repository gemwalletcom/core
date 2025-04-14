use chrono::{Duration, NaiveDateTime, Utc};
use primitives::Chain;

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub chain: Chain,
    pub timeout: u64,
    pub retry: u64,
}

impl ParserOptions {
    pub fn is_transaction_outdated(&self, transaction_created_at: NaiveDateTime) -> bool {
        Utc::now().naive_utc() - transaction_created_at > Duration::seconds(self.outdated_seconds())
    }

    pub fn outdated_seconds(&self) -> i64 {
        match self.chain {
            Chain::Bitcoin => 7_200,                // 2 hours
            Chain::Litecoin | Chain::Doge => 1_800, // 30 minutes
            _ => 900,                               // 15 minutes
        }
    }

    pub fn minimum_transfer_amount(&self) -> u64 {
        match self.chain {
            Chain::Tron | Chain::Xrp => 5_000,
            Chain::Stellar => 50_000,
            Chain::Polkadot => 10_000_000,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_is_transaction_outdated_positive() {
        let options = ParserOptions {
            chain: Chain::Bitcoin,
            timeout: 0,
            retry: 0,
        };
        let created_at = Utc::now() - Duration::seconds(options.outdated_seconds() + 1);
        assert!(options.is_transaction_outdated(created_at.naive_utc()));
    }

    #[test]
    fn test_is_transaction_outdated_negative() {
        let options = ParserOptions {
            chain: Chain::Bitcoin,
            timeout: 0,
            retry: 0,
        };
        let created_at = Utc::now() - Duration::seconds(options.outdated_seconds() - 1);
        assert!(!options.is_transaction_outdated(created_at.naive_utc()));
    }
}

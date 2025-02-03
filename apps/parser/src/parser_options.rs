use chrono::{DateTime, Duration, Utc};
use primitives::Chain;

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub chain: Chain,
    pub timeout: u64,
    pub retry: u64,
}

impl ParserOptions {
    pub fn is_transaction_outdated(&self, transaction_created_at: DateTime<Utc>) -> bool {
        Utc::now() - transaction_created_at > Duration::seconds(self.outdated_seconds())
    }

    pub fn outdated_seconds(&self) -> i64 {
        match self.chain {
            Chain::Bitcoin => 7_200,                // 2 hours
            Chain::Litecoin | Chain::Doge => 1_800, // 30 minutes
            _ => 900,                               // 15 minutes
        }
    }

    pub fn minimum_transfer_amount(&self) -> Option<u64> {
        match self.chain {
            Chain::Tron | Chain::Xrp | Chain::Stellar => Some(1_000),
            Chain::Polkadot => Some(100_000),
            _ => None,
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
        assert!(options.is_transaction_outdated(created_at));
    }

    #[test]
    fn test_is_transaction_outdated_negative() {
        let options = ParserOptions {
            chain: Chain::Bitcoin,
            timeout: 0,
            retry: 0,
        };
        let created_at = Utc::now() - Duration::seconds(options.outdated_seconds() - 1);
        assert!(!options.is_transaction_outdated(created_at));
    }
}

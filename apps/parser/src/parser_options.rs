use chrono::{DateTime, Duration, Utc};
use primitives::Chain;

#[derive(Default, Debug, Clone)]
pub struct ParserOptions {
    pub timeout: u64,
    pub retry: u64,
}

impl ParserOptions {
    pub fn is_transaction_outdated(&self, chain: Chain, transaction_created_at: DateTime<Utc>) -> bool {
        Utc::now() - transaction_created_at > Duration::milliseconds(self.outdated(chain))
    }

    pub fn outdated(&self, chain: Chain) -> i64 {
        match chain {
            Chain::Bitcoin => 7_200_000,                // 2 hours
            Chain::Litecoin | Chain::Doge => 1_800_000, // 30 minutes
            _ => 900_000,                               // 15 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_is_transaction_outdated_positive() {
        let options = ParserOptions::default();
        let created_at = Utc::now() - Duration::milliseconds(options.outdated(Chain::Bitcoin) + 1);
        assert!(options.is_transaction_outdated(Chain::Bitcoin, created_at));
    }

    #[test]
    fn test_is_transaction_outdated_negative() {
        let options = ParserOptions::default();
        let created_at = Utc::now() - Duration::milliseconds(options.outdated(Chain::Bitcoin) - 1);
        assert!(!options.is_transaction_outdated(Chain::Bitcoin, created_at));
    }
}

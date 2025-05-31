use chrono::{Duration, NaiveDateTime, Utc};
use primitives::Chain;

#[derive(Default)]
pub struct TransactionsConsumerConfig {}
impl TransactionsConsumerConfig {
    pub fn is_transaction_outdated(&self, transaction_created_at: NaiveDateTime, chain: Chain) -> bool {
        Utc::now().naive_utc() - transaction_created_at > Duration::seconds(self.outdated_seconds(chain))
    }

    pub fn outdated_seconds(&self, chain: Chain) -> i64 {
        match chain {
            Chain::Bitcoin => 7_200,                // 2 hours
            Chain::Litecoin | Chain::Doge => 1_800, // 30 minutes
            _ => 900,                               // 15 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_is_transaction_outdated_positive() {
        let options = TransactionsConsumerConfig::default();
        let created_at = Utc::now() - Duration::seconds(options.outdated_seconds(Chain::Bitcoin) + 1);
        assert!(options.is_transaction_outdated(created_at.naive_utc(), Chain::Bitcoin));
    }

    #[test]
    fn test_is_transaction_outdated_negative() {
        let options = TransactionsConsumerConfig::default();
        let created_at = Utc::now() - Duration::seconds(options.outdated_seconds(Chain::Bitcoin) - 1);
        assert!(!options.is_transaction_outdated(created_at.naive_utc(), Chain::Bitcoin));
    }
}

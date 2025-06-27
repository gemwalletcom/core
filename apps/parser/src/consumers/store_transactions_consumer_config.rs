use chrono::{Duration, NaiveDateTime, Utc};
use primitives::{Chain, Transaction, TransactionType};

#[derive(Default)]
pub struct StoreTransactionsConsumerConfig {}
impl StoreTransactionsConsumerConfig {
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

    pub fn minimum_transfer_amount(&self, chain: Chain) -> u64 {
        match chain {
            Chain::Tron | Chain::Xrp => 5_000,
            Chain::Stellar => 50_000,
            Chain::Polkadot => 10_000_000,
            Chain::Solana => 1_000,
            _ => 0,
        }
    }

    pub fn filter_transaction(&self, transaction: &Transaction) -> bool {
        self.filter_transaction_by_value(
            &transaction.value,
            &transaction.transaction_type,
            self.minimum_transfer_amount(transaction.asset_id.chain),
        )
    }

    pub fn filter_transaction_by_value(&self, value: &str, transaction_type: &TransactionType, minimum_transfer_amount: u64) -> bool {
        if *transaction_type == TransactionType::Transfer {
            if let Ok(value) = value.parse::<u64>() {
                return value >= minimum_transfer_amount;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, TransactionType};

    #[test]
    fn test_is_transaction_outdated_positive() {
        let options = StoreTransactionsConsumerConfig::default();
        let created_at = Utc::now() - Duration::seconds(options.outdated_seconds(Chain::Bitcoin) + 1);
        assert!(options.is_transaction_outdated(created_at.naive_utc(), Chain::Bitcoin));
    }

    #[test]
    fn test_is_transaction_outdated_negative() {
        let options = StoreTransactionsConsumerConfig::default();
        let created_at = Utc::now() - Duration::seconds(options.outdated_seconds(Chain::Bitcoin) - 1);
        assert!(!options.is_transaction_outdated(created_at.naive_utc(), Chain::Bitcoin));
    }

    #[test]
    fn test_filter_transaction() {
        let options = StoreTransactionsConsumerConfig::default();
        let test_cases = vec![
            ("1", TransactionType::Transfer, 0, true),
            ("500", TransactionType::Transfer, 1000, false),
            ("1000", TransactionType::Transfer, 1000, true),
            ("1500", TransactionType::Transfer, 1000, true),
            ("invalid", TransactionType::Transfer, 1000, true),
        ];

        for (transaction_value, transaction_type, minimum_transfer_amount, expected) in test_cases {
            assert_eq!(
                options.filter_transaction_by_value(transaction_value, &transaction_type, minimum_transfer_amount),
                expected
            );
        }
    }
}

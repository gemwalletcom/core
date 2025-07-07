use chrono::{Duration, NaiveDateTime, Utc};
use primitives::{AssetId, Chain, Transaction, TransactionType};

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

    pub fn minimum_transfer_amount(&self, chain: Chain) -> Option<u64> {
        match chain {
            Chain::Tron | Chain::Xrp => Some(10_000),
            Chain::Stellar => Some(50_000),
            Chain::Polkadot => Some(10_000_000),
            Chain::Solana => Some(10_000),
            _ => None,
        }
    }

    pub fn filter_transaction(&self, transaction: &Transaction) -> bool {
        self.filter_transaction_by_value(&transaction.value, &transaction.asset_id, &transaction.transaction_type)
    }

    pub fn filter_transaction_by_value(&self, value: &str, asset_id: &AssetId, transaction_type: &TransactionType) -> bool {
        if *transaction_type == TransactionType::Transfer && asset_id.is_native() {
            if let Ok(value) = value.parse::<u64>() {
                if let Some(minimum_transfer_amount) = self.minimum_transfer_amount(asset_id.chain) {
                    return value > minimum_transfer_amount;
                }
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
            ("1", AssetId::from_chain(Chain::Ethereum), TransactionType::Transfer, true),
            ("10000", AssetId::from_chain(Chain::Solana), TransactionType::Transfer, false),
            ("10000", AssetId::from(Chain::Solana, Some("1".to_string())), TransactionType::Transfer, true),
            ("10001", AssetId::from_chain(Chain::Solana), TransactionType::Transfer, true),
            ("1001", AssetId::from_chain(Chain::Ethereum), TransactionType::Transfer, true),
            ("1500", AssetId::from_chain(Chain::Ethereum), TransactionType::Transfer, true),
            ("invalid", AssetId::from_chain(Chain::Ethereum), TransactionType::Transfer, true),
        ];

        for (transaction_value, asset_id, transaction_type, expected) in test_cases {
            assert_eq!(options.filter_transaction_by_value(transaction_value, &asset_id, &transaction_type), expected);
        }
    }
}

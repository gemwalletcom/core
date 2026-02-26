use std::collections::HashSet;
use std::time::Duration;

use chrono::NaiveDateTime;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, Chain, Price, Transaction, TransactionState, TransactionType};

pub struct StoreTransactionsConsumerConfig {
    pub swap_outdated_timeout: Duration,
    pub outdated_block_count: u64,
    pub outdated_min_timeout: Duration,
    pub min_amount_usd: f64,
}

impl StoreTransactionsConsumerConfig {
    pub fn is_transaction_outdated(&self, transaction_created_at: NaiveDateTime, chain: Chain, transaction_type: TransactionType) -> bool {
        let elapsed = (chrono::Utc::now().naive_utc() - transaction_created_at).to_std().unwrap_or_default();
        elapsed > self.outdated_timeout(chain, transaction_type)
    }

    fn outdated_timeout(&self, chain: Chain, transaction_type: TransactionType) -> Duration {
        if transaction_type == TransactionType::Swap {
            return self.swap_outdated_timeout;
        }
        let block_time_secs = chain.block_time() as u64 / 1000;
        Duration::from_secs(block_time_secs * self.outdated_block_count).max(self.outdated_min_timeout)
    }

    pub fn should_notify_transaction(&self, transaction: &Transaction, is_notify_devices: bool, vault_addresses: &HashSet<String>) -> bool {
        is_notify_devices
            && transaction.state != TransactionState::InTransit
            && !vault_addresses.contains(&transaction.from)
            && !self.is_transaction_outdated(transaction.created_at.naive_utc(), transaction.asset_id.chain, transaction.transaction_type.clone())
    }

    pub fn is_transaction_insufficient_amount(&self, transaction: &Transaction, asset: &Asset, price: Option<Price>, min_amount: f64) -> bool {
        if transaction.transaction_type == TransactionType::Transfer
            && let Ok(amount) = BigNumberFormatter::value_as_f64(&transaction.value, asset.decimals as u32)
            && let Some(price) = price
        {
            return amount * price.price <= min_amount;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, TransactionType};

    impl StoreTransactionsConsumerConfig {
        fn mock() -> Self {
            Self {
                swap_outdated_timeout: Duration::from_secs(7_200),
                outdated_block_count: 12,
                outdated_min_timeout: Duration::from_secs(900),
                min_amount_usd: 0.01,
            }
        }
    }

    #[test]
    fn test_is_transaction_outdated_positive() {
        let config = StoreTransactionsConsumerConfig::mock();
        let timeout = config.outdated_timeout(Chain::Bitcoin, TransactionType::Transfer);
        let created_at = chrono::Utc::now() - chrono::Duration::from_std(timeout).unwrap() - chrono::Duration::seconds(1);
        assert!(config.is_transaction_outdated(created_at.naive_utc(), Chain::Bitcoin, TransactionType::Transfer));
    }

    #[test]
    fn test_is_transaction_outdated_negative() {
        let config = StoreTransactionsConsumerConfig::mock();
        let timeout = config.outdated_timeout(Chain::Bitcoin, TransactionType::Transfer);
        let created_at = chrono::Utc::now() - chrono::Duration::from_std(timeout).unwrap() + chrono::Duration::seconds(1);
        assert!(!config.is_transaction_outdated(created_at.naive_utc(), Chain::Bitcoin, TransactionType::Transfer));
    }

    #[test]
    fn test_is_swap_outdated_positive() {
        let config = StoreTransactionsConsumerConfig::mock();
        let timeout = config.outdated_timeout(Chain::Ethereum, TransactionType::Swap);
        let created_at = chrono::Utc::now() - chrono::Duration::from_std(timeout).unwrap() - chrono::Duration::seconds(1);
        assert!(config.is_transaction_outdated(created_at.naive_utc(), Chain::Ethereum, TransactionType::Swap));
    }

    #[test]
    fn test_is_swap_outdated_negative() {
        let config = StoreTransactionsConsumerConfig::mock();
        let timeout = config.outdated_timeout(Chain::Ethereum, TransactionType::Swap);
        let created_at = chrono::Utc::now() - chrono::Duration::from_std(timeout).unwrap() + chrono::Duration::seconds(1);
        assert!(!config.is_transaction_outdated(created_at.naive_utc(), Chain::Ethereum, TransactionType::Swap));
    }

    #[test]
    fn test_is_transaction_insufficient_amount() {
        use chrono::Utc;
        use primitives::AssetId;

        let config = StoreTransactionsConsumerConfig::mock();

        let token_asset = Asset::mock_erc20();
        let native_asset = Asset::mock_btc();

        let price_high = Some(Price::new(1.0, 0.0, Utc::now()));
        let price_low = Some(Price::new(0.005, 0.0, Utc::now()));

        let transaction_transfer = Transaction::mock_with_params(
            AssetId::from(Chain::Ethereum, Some("0xA0b86a33E6441066d64bb38954e41F6b4b925c59".to_string())),
            TransactionType::Transfer,
            "100000".to_string(),
        );

        let transaction_swap = Transaction::mock_with_params(
            AssetId::from(Chain::Ethereum, Some("0xA0b86a33E6441066d64bb38954e41F6b4b925c59".to_string())),
            TransactionType::Swap,
            "100000".to_string(),
        );

        let test_cases = vec![
            (transaction_transfer.clone(), &token_asset, price_high, 0.01, false),
            (transaction_transfer.clone(), &token_asset, price_low, 0.01, true),
            (transaction_transfer.clone(), &token_asset, price_high, 0.5, true),
            (transaction_transfer.clone(), &native_asset, price_low, 0.01, true),
            (transaction_transfer.clone(), &token_asset, None, 0.01, false),
            (transaction_swap.clone(), &token_asset, price_low, 0.01, false),
        ];

        for (transaction, asset, price, min_amount, expected) in test_cases {
            assert_eq!(config.is_transaction_insufficient_amount(&transaction, asset, price, min_amount), expected);
        }
    }

    #[test]
    fn test_should_notify_transaction() {
        let config = StoreTransactionsConsumerConfig::mock();
        let empty = HashSet::new();

        assert!(config.should_notify_transaction(&Transaction::mock(), true, &empty));
    }

    #[test]
    fn test_should_notify_transaction_in_transit() {
        let config = StoreTransactionsConsumerConfig::mock();
        let empty = HashSet::new();
        let tx = Transaction {
            state: TransactionState::InTransit,
            ..Transaction::mock()
        };
        assert!(!config.should_notify_transaction(&tx, true, &empty));
    }

    #[test]
    fn test_should_notify_transaction_no_devices() {
        let config = StoreTransactionsConsumerConfig::mock();
        let empty = HashSet::new();
        assert!(!config.should_notify_transaction(&Transaction::mock(), false, &empty));
    }

    #[test]
    fn test_should_notify_transaction_from_vault() {
        let config = StoreTransactionsConsumerConfig::mock();
        let tx = Transaction::mock();
        let vault_addresses = HashSet::from([tx.from.clone()]);
        assert!(!config.should_notify_transaction(&tx, true, &vault_addresses));
    }
}

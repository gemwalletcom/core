use chrono::{Duration, NaiveDateTime, Utc};
use number_formatter::BigNumberFormatter;
use primitives::{Asset, Chain, Price, Transaction, TransactionType};

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

    pub fn is_transaction_sufficient_amount(&self, transaction: &Transaction, asset: Option<Asset>, price: Option<Price>, min_amount: f64) -> bool {
        if let Some(asset) = asset
            && transaction.transaction_type == TransactionType::Transfer
            && let Ok(amount) = BigNumberFormatter::value_as_f64(&transaction.value, asset.decimals as u32)
            && let Some(price) = price
        {
            return amount * price.price > min_amount;
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
    fn test_is_transaction_sufficient_amount() {
        use chrono::Utc;
        use primitives::AssetId;

        let options = StoreTransactionsConsumerConfig::default();

        let token_asset = Some(Asset::mock_erc20());
        let native_asset = Some(Asset::mock_btc());

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
            (transaction_transfer.clone(), token_asset.clone(), price_high, 0.01, true),
            (transaction_transfer.clone(), token_asset.clone(), price_low, 0.01, false),
            (transaction_transfer.clone(), token_asset.clone(), price_high, 0.5, false),
            (transaction_transfer.clone(), native_asset.clone(), price_low, 0.01, false),
            (transaction_transfer.clone(), None, price_high, 0.01, true),
            (transaction_transfer.clone(), token_asset.clone(), None, 0.01, true),
            (transaction_swap.clone(), token_asset.clone(), price_low, 0.01, true),
        ];

        for (transaction, asset, price, min_amount, expected) in test_cases {
            assert_eq!(options.is_transaction_sufficient_amount(&transaction, asset, price, min_amount), expected);
        }
    }
}

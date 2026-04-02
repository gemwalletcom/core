use primitives::date_ext::NaiveDateTimeExt;
use primitives::{FiatProviderName, FiatQuoteType, FiatTransaction, FiatTransactionData, FiatTransactionStatus};

pub fn fiat_transaction_info(mut transaction: FiatTransaction) -> FiatTransactionData {
    if transaction.status == FiatTransactionStatus::Pending && transaction.created_at.naive_utc().is_older_than_days(1) {
        transaction.status = FiatTransactionStatus::Unknown;
    }

    let details_url = match transaction.status {
        FiatTransactionStatus::Unknown => None,
        _ => details_url(&transaction.provider, &transaction.transaction_type, transaction.provider_transaction_id.as_deref()),
    };

    FiatTransactionData { transaction, details_url }
}

fn details_url(provider: &FiatProviderName, transaction_type: &FiatQuoteType, provider_transaction_id: Option<&str>) -> Option<String> {
    let provider_transaction_id = provider_transaction_id?;

    match provider {
        FiatProviderName::MoonPay => match transaction_type {
            FiatQuoteType::Buy => Some(format!("https://buy.moonpay.com/v2/transaction-tracker?transactionId={provider_transaction_id}")),
            FiatQuoteType::Sell => Some(format!("https://sell.moonpay.com/v2/transaction-tracker?transactionId={provider_transaction_id}")),
        },
        FiatProviderName::Mercuryo => None,
        FiatProviderName::Transak => None,
        FiatProviderName::Banxa => Some(format!("https://gemwallet.banxa.com/status/{provider_transaction_id}")),
        FiatProviderName::Paybis => None,
        FiatProviderName::Flashnet => Some(format!("https://orchestra.flashnet.xyz/explorer/{provider_transaction_id}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{details_url, fiat_transaction_info};
    use primitives::chrono::{Duration, Utc};
    use primitives::{FiatProviderName, FiatQuoteType, FiatTransactionStatus};

    #[test]
    fn details_url_returns_expected_values() {
        let cases = [
            (
                FiatProviderName::MoonPay,
                FiatQuoteType::Buy,
                Some("tx_123"),
                Some("https://buy.moonpay.com/v2/transaction-tracker?transactionId=tx_123"),
            ),
            (
                FiatProviderName::MoonPay,
                FiatQuoteType::Sell,
                Some("tx_123"),
                Some("https://sell.moonpay.com/v2/transaction-tracker?transactionId=tx_123"),
            ),
            (FiatProviderName::MoonPay, FiatQuoteType::Buy, None, None),
            (
                FiatProviderName::Flashnet,
                FiatQuoteType::Buy,
                Some("ord_123"),
                Some("https://orchestra.flashnet.xyz/explorer/ord_123"),
            ),
            (FiatProviderName::Mercuryo, FiatQuoteType::Buy, Some("tx_123"), None),
            (FiatProviderName::Transak, FiatQuoteType::Buy, Some("tx_123"), None),
            (
                FiatProviderName::Banxa,
                FiatQuoteType::Buy,
                Some("tx_123"),
                Some("https://gemwallet.banxa.com/status/tx_123"),
            ),
            (FiatProviderName::Paybis, FiatQuoteType::Sell, Some("PB123"), None),
        ];

        for (provider, transaction_type, transaction_id, expected) in cases {
            let result = details_url(&provider, &transaction_type, transaction_id);
            assert_eq!(result.as_deref(), expected);
        }
    }

    #[test]
    fn from_primitive_sets_details_url_on_render() {
        let transaction = primitives::FiatTransaction {
            transaction_type: FiatQuoteType::Buy,
            status: FiatTransactionStatus::Complete,
            ..primitives::FiatTransaction::mock()
        };

        let rendered = fiat_transaction_info(transaction);

        assert_eq!(
            rendered.details_url,
            Some("https://buy.moonpay.com/v2/transaction-tracker?transactionId=tx_123".to_string())
        );
    }

    #[test]
    fn pending_older_than_one_day_becomes_unknown() {
        let transaction = primitives::FiatTransaction {
            status: FiatTransactionStatus::Pending,
            created_at: Utc::now() - Duration::days(2),
            ..primitives::FiatTransaction::mock()
        };

        let result = fiat_transaction_info(transaction);
        assert_eq!(result.transaction.status, FiatTransactionStatus::Unknown);
        assert_eq!(result.details_url, None);
    }

    #[test]
    fn recent_pending_stays_pending() {
        let transaction = primitives::FiatTransaction {
            status: FiatTransactionStatus::Pending,
            created_at: Utc::now(),
            ..primitives::FiatTransaction::mock()
        };

        let result = fiat_transaction_info(transaction);
        assert_eq!(result.transaction.status, FiatTransactionStatus::Pending);
    }
}

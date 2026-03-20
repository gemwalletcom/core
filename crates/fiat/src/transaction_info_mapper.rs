use primitives::{FiatProviderName, FiatQuoteType, FiatTransaction, FiatTransactionInfo};

pub fn fiat_transaction_info(transaction: FiatTransaction) -> FiatTransactionInfo {
    let details_url = details_url(&transaction.provider_id, &transaction.transaction_type, &transaction.provider_transaction_id);

    FiatTransactionInfo { transaction, details_url }
}

fn details_url(provider_id: &FiatProviderName, transaction_type: &FiatQuoteType, provider_transaction_id: &str) -> Option<String> {
    match provider_id {
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
    use primitives::{FiatProviderName, FiatQuoteType};

    #[test]
    fn details_url_returns_expected_values() {
        let cases = [
            (
                FiatProviderName::MoonPay,
                FiatQuoteType::Buy,
                "tx_123",
                Some("https://buy.moonpay.com/v2/transaction-tracker?transactionId=tx_123"),
            ),
            (
                FiatProviderName::MoonPay,
                FiatQuoteType::Sell,
                "tx_123",
                Some("https://sell.moonpay.com/v2/transaction-tracker?transactionId=tx_123"),
            ),
            (
                FiatProviderName::Flashnet,
                FiatQuoteType::Buy,
                "ord_123",
                Some("https://orchestra.flashnet.xyz/explorer/ord_123"),
            ),
            (FiatProviderName::Mercuryo, FiatQuoteType::Buy, "tx_123", None),
            (FiatProviderName::Transak, FiatQuoteType::Buy, "tx_123", None),
            (FiatProviderName::Banxa, FiatQuoteType::Sell, "123", Some("https://gemwallet.banxa.com/status/123")),
            (FiatProviderName::Paybis, FiatQuoteType::Sell, "PB123", None),
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
            ..primitives::FiatTransaction::mock()
        };

        let rendered = fiat_transaction_info(transaction);

        assert_eq!(
            rendered.details_url,
            Some("https://buy.moonpay.com/v2/transaction-tracker?transactionId=tx_123".to_string())
        );
    }
}

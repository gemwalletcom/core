use primitives::{AssetId, Chain, FiatQuoteType, FiatTransaction, FiatTransactionStatus};

use super::{
    client::PaybisClient,
    models::{Currency, PaybisTransaction, PaybisWebhook},
};

pub fn map_asset_id(currency: Currency) -> Option<AssetId> {
    if !currency.is_crypto() {
        return None;
    }
    map_symbol_to_asset_id(&currency.code)
}

pub fn map_symbol_to_asset_id(symbol: &str) -> Option<AssetId> {
    match symbol {
        "BTC" => Some(AssetId::from_chain(Chain::Bitcoin)),
        "BCH" => Some(AssetId::from_chain(Chain::BitcoinCash)),
        "ETH" => Some(AssetId::from_chain(Chain::Ethereum)),
        "XRP" => Some(AssetId::from_chain(Chain::Xrp)),
        "SOL" => Some(AssetId::from_chain(Chain::Solana)),
        "XLM" => Some(AssetId::from_chain(Chain::Stellar)),
        "TRX" => Some(AssetId::from_chain(Chain::Tron)),
        "ADA" => Some(AssetId::from_chain(Chain::Cardano)),
        "OP" => Some(AssetId::from_chain(Chain::Optimism)),
        "LTC" => Some(AssetId::from_chain(Chain::Litecoin)),
        "DOT" => Some(AssetId::from_chain(Chain::Polkadot)),
        "CELO" => Some(AssetId::from_chain(Chain::Celo)),
        "TON" => Some(AssetId::from_chain(Chain::Ton)),
        "DOGE" => Some(AssetId::from_chain(Chain::Doge)),

        "ARB" => Some(AssetId::from_chain(Chain::Arbitrum)),
        "AVAXC" => Some(AssetId::from_chain(Chain::AvalancheC)),

        "ETH-BASE" => Some(AssetId::from_chain(Chain::Base)),
        "USDC-BASE" => Some(AssetId::from(Chain::Base, Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()))),

        "POL" => Some(AssetId::from_chain(Chain::Polygon)),
        "USDC-POLYGON" => Some(AssetId::from(Chain::Polygon, Some("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359".to_string()))),
        "USDT-POLYGON" => Some(AssetId::from(Chain::Polygon, Some("0xc2132D05D31c914a87C6611C10748AEb04B58e8F".to_string()))),

        "USDC-SOL" => Some(AssetId::from(Chain::Solana, Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()))),
        "USDT-SOL" => Some(AssetId::from(Chain::Solana, Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string()))),

        "USDT-TRC20" => Some(AssetId::from(Chain::Tron, Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()))),

        "BNBSC" => Some(AssetId::from_chain(Chain::SmartChain)),

        "USDC" => Some(AssetId::from(Chain::Ethereum, Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()))),
        "USDT" => Some(AssetId::from(Chain::Ethereum, Some("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()))),

        _ => None,
    }
}

pub fn map_order_from_response(transaction: PaybisTransaction) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let asset_id = map_symbol_to_asset_id(&transaction.crypto_currency);

    let status = match transaction.status.as_str() {
        "pending" | "confirming" => FiatTransactionStatus::Pending,
        "failed" | "cancelled" | "canceled" => FiatTransactionStatus::Failed,
        "completed" | "success" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown(transaction.status.clone()),
    };

    let transaction_type = FiatQuoteType::Buy; // TODO: Determine from API response

    Ok(FiatTransaction {
        asset_id,
        transaction_type,
        symbol: transaction.crypto_currency,
        provider_id: PaybisClient::NAME.id(),
        provider_transaction_id: transaction.id,
        status,
        country: transaction.country,
        fiat_amount: transaction.fiat_amount,
        fiat_currency: transaction.fiat_currency.to_uppercase(),
        transaction_hash: transaction.transaction_hash,
        address: transaction.wallet_address,
        fee_provider: transaction.service_fee,
        fee_network: transaction.network_fee,
        fee_partner: transaction.partner_fee,
    })
}

pub fn map_webhook_order_id(data: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_value::<PaybisWebhook>(data)?.data.transaction.invoice)
}

pub fn map_webhook_to_transaction(data: serde_json::Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let webhook = serde_json::from_value::<PaybisWebhook>(data)?.data;

    let transaction_type = match webhook.transaction.flow.as_str() {
        "buyCrypto" => FiatQuoteType::Buy,
        "sellCrypto" => FiatQuoteType::Sell,
        _ => FiatQuoteType::Buy,
    };

    let status = match webhook.transaction.status.as_str() {
        "started" | "pending" | "confirming" => FiatTransactionStatus::Pending,
        "completed" | "success" => FiatTransactionStatus::Complete,
        "failed" | "cancelled" | "canceled" => FiatTransactionStatus::Failed,
        _ => FiatTransactionStatus::Unknown(webhook.transaction.status.clone()),
    };

    let crypto_currency = &webhook.amount_to.currency;
    let fiat_currency = &webhook.amount_from.currency;
    let asset_id = map_symbol_to_asset_id(crypto_currency);

    let fiat_amount: f64 = webhook.amount_from.amount.parse().unwrap_or(0.0);

    Ok(FiatTransaction {
        asset_id,
        transaction_type,
        symbol: crypto_currency.clone(),
        provider_id: PaybisClient::NAME.id(),
        provider_transaction_id: webhook.transaction.invoice,
        status,
        country: webhook
            .payment
            .as_ref()
            .and_then(|p| p.card.as_ref())
            .map(|c| c.billing_address.country.code.clone()),
        fiat_amount,
        fiat_currency: fiat_currency.to_uppercase(),
        transaction_hash: webhook.payout.as_ref().and_then(|p| p.transaction_hash.clone()),
        address: webhook.payout.as_ref().map(|p| p.destination_wallet_address.clone()),
        fee_provider: None,
        fee_network: None,
        fee_partner: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_asset_id() {
        assert_eq!(
            map_asset_id(Currency {
                code: "ETH".to_string(),
                blockchain_name: Some("ethereum".to_string()),
            }),
            Some(AssetId::from_chain(Chain::Ethereum))
        );

        assert_eq!(
            map_asset_id(Currency {
                code: "BTC".to_string(),
                blockchain_name: Some("bitcoin".to_string()),
            }),
            Some(AssetId::from_chain(Chain::Bitcoin))
        );

        assert_eq!(
            map_asset_id(Currency {
                code: "UNKNOWN".to_string(),
                blockchain_name: Some("unknown-chain".to_string()),
            }),
            None
        );

        assert_eq!(
            map_asset_id(Currency {
                code: "USD".to_string(),
                blockchain_name: None,
            }),
            None
        );
    }

    #[test]
    fn test_map_symbol_to_asset_id_coins() {
        assert_eq!(map_symbol_to_asset_id("BTC"), Some(AssetId::from_chain(Chain::Bitcoin)));
        assert_eq!(map_symbol_to_asset_id("ETH"), Some(AssetId::from_chain(Chain::Ethereum)));
        assert_eq!(map_symbol_to_asset_id("TRX"), Some(AssetId::from_chain(Chain::Tron)));
        assert_eq!(map_symbol_to_asset_id("XRP"), Some(AssetId::from_chain(Chain::Xrp)));
        assert_eq!(map_symbol_to_asset_id("SOL"), Some(AssetId::from_chain(Chain::Solana)));
        assert_eq!(map_symbol_to_asset_id("ADA"), Some(AssetId::from_chain(Chain::Cardano)));
        assert_eq!(map_symbol_to_asset_id("DOT"), Some(AssetId::from_chain(Chain::Polkadot)));
        assert_eq!(map_symbol_to_asset_id("TON"), Some(AssetId::from_chain(Chain::Ton)));
        assert_eq!(map_symbol_to_asset_id("DOGE"), Some(AssetId::from_chain(Chain::Doge)));

        assert_eq!(map_symbol_to_asset_id("ARB"), Some(AssetId::from_chain(Chain::Arbitrum)));
        assert_eq!(map_symbol_to_asset_id("AVAXC"), Some(AssetId::from_chain(Chain::AvalancheC)));
        assert_eq!(map_symbol_to_asset_id("POL"), Some(AssetId::from_chain(Chain::Polygon)));
        assert_eq!(map_symbol_to_asset_id("BNBSC"), Some(AssetId::from_chain(Chain::SmartChain)));

        assert_eq!(map_symbol_to_asset_id("ETH-BASE"), Some(AssetId::from_chain(Chain::Base)));

        assert_eq!(map_symbol_to_asset_id("UNKNOWN"), None);
    }

    #[test]
    fn test_map_symbol_to_asset_id_tokens() {
        let token_tests = vec![
            ("USDC", Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            ("USDC-BASE", Chain::Base, "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
            ("USDC-SOL", Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            ("USDT", Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ("USDT-TRC20", Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"),
        ];

        for (symbol, expected_chain, expected_token_id) in token_tests {
            let result = map_symbol_to_asset_id(symbol);
            let expected = Some(AssetId::from(expected_chain, Some(expected_token_id.to_string())));
            assert_eq!(result, expected, "Failed for symbol: {}", symbol);
        }
    }

    #[test]
    fn test_map_webhook_order_id() {
        let webhook_data: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started.json")).unwrap();

        let order_id = map_webhook_order_id(webhook_data.clone()).unwrap();
        assert_eq!(order_id, "PB21095868675TX1");
    }

    #[test]
    fn test_map_webhook_to_transaction() {
        let webhook_data: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started.json")).unwrap();

        let transaction = map_webhook_to_transaction(webhook_data.clone()).unwrap();
        assert_eq!(transaction.provider_transaction_id, "PB21095868675TX1");
        assert_eq!(transaction.symbol, "SOL");
        assert_eq!(transaction.fiat_currency, "USD");
        assert_eq!(transaction.fiat_amount, 50.0);
        assert!(matches!(transaction.transaction_type, FiatQuoteType::Buy));
        assert!(matches!(transaction.status, FiatTransactionStatus::Pending));
        assert_eq!(transaction.country, Some("US".to_string()));
        assert_eq!(transaction.address, Some("test123".to_string()));
        assert_eq!(transaction.transaction_hash, None);
    }
}

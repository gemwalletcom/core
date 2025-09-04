use primitives::{AssetId, Chain, FiatQuoteType, FiatTransaction, FiatTransactionStatus};
use streamer::FiatWebhook;

use crate::providers::paybis::models::PaybisWebhook;

use super::{
    client::PaybisClient,
    models::{Currency, PaybisWebhookData},
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

pub fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "started" | "pending" | "confirming" | "payment-authorized" => FiatTransactionStatus::Pending,
        "completed" | "success" => FiatTransactionStatus::Complete,
        "failed" | "cancelled" | "canceled" | "rejected" => FiatTransactionStatus::Failed,
        _ => FiatTransactionStatus::Unknown(status.to_string()),
    }
}

pub fn map_process_webhook(data: serde_json::Value) -> FiatWebhook {
    match serde_json::from_value::<PaybisWebhook>(data) {
        Ok(webhook) => map_webhook_data(webhook.data),
        Err(_) => FiatWebhook::None,
    }
}

pub fn map_webhook_data(webhook_data: PaybisWebhookData) -> FiatWebhook {
    FiatWebhook::Transaction(FiatTransaction {
        asset_id: map_symbol_to_asset_id(&webhook_data.amount_to.currency),
        transaction_type: match webhook_data.transaction.flow.as_str() {
            "buyCrypto" => FiatQuoteType::Buy,
            "sellCrypto" => FiatQuoteType::Sell,
            _ => FiatQuoteType::Buy,
        },
        symbol: webhook_data.amount_to.currency,
        provider_id: PaybisClient::NAME.id(),
        provider_transaction_id: webhook_data.transaction.invoice,
        status: map_status(&webhook_data.transaction.status),
        country: webhook_data
            .payment
            .as_ref()
            .and_then(|p| p.card.as_ref())
            .map(|c| c.billing_address.country.code.clone()),
        fiat_amount: webhook_data.amount_from.amount.parse().unwrap_or(0.0),
        fiat_currency: webhook_data.amount_from.currency.to_uppercase(),
        transaction_hash: webhook_data.payout.as_ref().and_then(|p| p.transaction_hash.clone()),
        address: webhook_data.payout.as_ref().and_then(|p| p.destination_wallet_address.clone()),
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
    fn test_map_process_webhook() {
        let webhook_json: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started.json")).unwrap();

        let result = map_process_webhook(webhook_json);
        if let FiatWebhook::Transaction(transaction) = result {
            assert_eq!(transaction.provider_transaction_id, "PB21095868675TX1");
            assert_eq!(transaction.symbol, "SOL");
            assert_eq!(transaction.fiat_currency, "USD");
        } else {
            panic!("Expected FiatWebhook::Transaction variant");
        }
    }

    #[test]
    fn test_map_process_webhook_with_payment() {
        let webhook_json: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started.json")).unwrap();

        let result = map_process_webhook(webhook_json);
        if let FiatWebhook::Transaction(transaction) = result {
            assert_eq!(transaction.provider_transaction_id, "PB21095868675TX1");
            assert_eq!(transaction.symbol, "SOL");
            assert_eq!(transaction.fiat_currency, "USD");
            assert_eq!(transaction.fiat_amount, 50.0);
            assert!(matches!(transaction.transaction_type, FiatQuoteType::Buy));
            assert!(matches!(transaction.status, FiatTransactionStatus::Pending));
            assert_eq!(transaction.country, Some("US".to_string()));
            assert_eq!(transaction.address, Some("test123".to_string()));
            assert_eq!(transaction.transaction_hash, None);
        } else {
            panic!("Expected FiatWebhook::Transaction variant");
        }
    }

    #[test]
    fn test_map_process_webhook_no_payment() {
        let webhook_json: serde_json::Value =
            serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started_no_payment.json")).unwrap();

        let result = map_process_webhook(webhook_json);
        if let FiatWebhook::Transaction(transaction) = result {
            assert_eq!(transaction.provider_transaction_id, "PB25095868675TX8");
            assert_eq!(transaction.symbol, "SOL");
            assert_eq!(transaction.fiat_currency, "USD");
            assert_eq!(transaction.country, None);
            assert_eq!(transaction.address, None);
        } else {
            panic!("Expected FiatWebhook::Transaction variant");
        }
    }

    #[test]
    fn test_verification_webhook_maps_to_none() {
        let data: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_no_changes.json")).unwrap();

        let result = map_process_webhook(data);
        assert!(matches!(result, FiatWebhook::None), "Verification webhooks should map to FiatWebhook::None");
    }
}
